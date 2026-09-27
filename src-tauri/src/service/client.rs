use super::watchdog;
use crate::app_state::{AppSnapshot, AppState, ForwardStatus, NetworkMode};
use crate::easytier::args::{ClientArgs, FALLBACK_RPC_CLIENT};
use crate::easytier::binaries;
use crate::easytier::cli::{self, Proto};
use crate::easytier::process;
use crate::mc::fake_server;
use crate::ports::allocate_rpc_port;
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub struct ForwardRule {
    pub proto: Proto,
    pub local: String,
    pub remote: String,
    /// v4 转发是127.0.0.1 加入路径的必需品；v6 仅作 IPv6 目标兜底，失败不阻断启动。
    pub required: bool,
}

pub fn forward_rules(port: u16) -> Vec<ForwardRule> {
    let remote = format!("{}:{port}", crate::easytier::args::SERVER_IPV4);
    let mut rules = Vec::new();
    for local in [format!("0.0.0.0:{port}"), format!("[::]:{port}")] {
        let required = local.starts_with("0.0.0.0");
        for proto in [Proto::Tcp, Proto::Udp] {
            rules.push(ForwardRule {
                proto,
                local: local.clone(),
                remote: remote.clone(),
                required,
            });
        }
    }
    rules
}

pub fn retry<T>(attempts: u32, mut f: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    let mut last = String::new();
    for i in 0..attempts {
        match f() {
            Ok(v) => return Ok(v),
            Err(e) => {
                last = e;
                if i + 1 < attempts {
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
    }
    Err(last)
}

pub fn create_client(
    state: &Arc<AppState>,
    emit: impl Fn(&str, &AppSnapshot) + Send + Sync + 'static,
    emit_log: impl Fn(String) + Send + Sync + 'static,
    name: String,
    secret: String,
    nodes: Vec<String>,
    mc_port: u16,
    stop_flag: Arc<AtomicBool>,
) -> Result<crate::app_state::AppSnapshot, String> {
    if name.trim().is_empty() || secret.trim().is_empty() {
        return Err("network-name / network-secret 不能为空".into());
    }
    if nodes.is_empty() {
        return Err("至少需要一个公网节点".into());
    }
    let flow = state.begin(NetworkMode::ClientConnecting)?;
    let worker_state = Arc::clone(state);
    let emit: watchdog::EmitFn = Arc::new(emit);

    thread::spawn(move || {
        let rpc_port = allocate_rpc_port(FALLBACK_RPC_CLIENT);
        let args = ClientArgs {
            name: name.clone(),
            secret: secret.clone(),
            public_nodes: nodes,
            rpc_port,
        };
        let paths = binaries::acquire();
        let log = crate::log_buffer();
        let Ok(mut child) = process::spawn(paths.core.as_path(), &args.to_command_line(), rpc_port, log) else {
            worker_state.finish_error(flow, "easytier-core 启动失败".into());
            emit("network-status", &worker_state.snapshot());
            return;
        };

        if !worker_state.update(flow, |s| {
            s.name = Some(name.clone());
            s.secret = Some(secret.clone());
            s.mc_port = Some(mc_port);
            s.rpc_port = Some(rpc_port);
        }) {
            child.kill();
            return;
        }
        emit("network-status", &worker_state.snapshot());

        thread::sleep(Duration::from_secs(3));
        let cli_path = paths.cli.clone();
        let peer_check = retry(5, || {
            if stop_flag.load(Ordering::SeqCst) {
                return Err("已取消".into());
            }
            cli::check_peer(&cli_path, rpc_port).map(|_| ())
        });
        if let Err(e) = peer_check {
            child.kill();
            worker_state.finish_error(flow, format!("RPC 连通性验证失败: {e}"));
            emit("network-status", &worker_state.snapshot());
            return;
        }
        emit_log("peer check ok".into());

        let mut forwards = Vec::new();
        for rule in forward_rules(mc_port) {
            let (local, remote) = (rule.local.clone(), rule.remote.clone());
            let attempts = if rule.required { 3 } else { 1 };
            let result = retry(attempts, || {
                cli::add_port_forward(&cli_path, rpc_port, rule.proto, &local, &remote)
            });
            forwards.push(ForwardStatus {
                proto: match rule.proto { Proto::Tcp => "tcp".into(), Proto::Udp => "udp".into() },
                local,
                remote,
                ok: result.is_ok(),
            });
            if result.is_err() && rule.required {
                child.kill();
                worker_state.update(flow, |s| s.forwards = forwards.clone());
                worker_state.finish_error(flow, "端口转发添加失败".into());
                emit("network-status", &worker_state.snapshot());
                return;
            }
        }

        // MC 通过127.0.0.1:{mc_port} 进入（fake_server 的公告源地址），
        // 本地监听没起来之前不广播，避免出现「看得到进不去」的房间。
        let probe = SocketAddr::from((Ipv4Addr::LOCALHOST, mc_port));
        let mut accepting = false;
        for _ in 0..10 {
            if stop_flag.load(Ordering::SeqCst) {
                child.kill();
                worker_state.update(flow, |s| s.forwards = forwards.clone());
                worker_state.finish_error(flow, "已取消".into());
                emit("network-status", &worker_state.snapshot());
                return;
            }
            if TcpStream::connect_timeout(&probe, Duration::from_millis(500)).is_ok() {
                accepting = true;
                break;
            }
            thread::sleep(Duration::from_millis(500));
        }
        if !accepting {
            child.kill();
            worker_state.update(flow, |s| s.forwards = forwards.clone());
            worker_state.finish_error(flow, format!("端口转发未就绪：127.0.0.1:{mc_port} 拒绝连接"));
            emit("network-status", &worker_state.snapshot());
            return;
        }
        emit_log(format!("local forward accepting on 127.0.0.1:{mc_port}"));

        let server = fake_server::start(args.name.clone(), mc_port);
        let server_slot: Arc<Mutex<Option<fake_server::FakeServer>>> = Arc::new(Mutex::new(Some(server)));
        crate::set_fake_server_slot(server_slot);

        if !worker_state.update(flow, |s| {
            s.mode = NetworkMode::ClientRunning;
            s.forwards = forwards;
        }) {
            crate::take_fake_server();
            child.kill();
            return;
        }
        crate::set_instance(child);
        watchdog::watch(worker_state.clone(), flow, Arc::clone(&emit), Arc::clone(&stop_flag));
        emit_log("client running with lan broadcast".into());
        emit("network-status", &worker_state.snapshot());
    });

    Ok(state.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_rules_cover_ipv4_and_ipv6_with_shared_target() {
        let rules = forward_rules(25565);
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].local, "0.0.0.0:25565");
        assert_eq!(rules[1].local, "0.0.0.0:25565");
        assert_eq!(rules[2].local, "[::]:25565");
        assert_eq!(rules[3].local, "[::]:25565");
        for rule in &rules {
            assert_eq!(rule.remote, "10.144.144.1:25565");
        }
        assert!(rules[0].required && rules[1].required);
        assert!(!rules[2].required && !rules[3].required);

        let other = forward_rules(25566);
        assert_eq!(other[0].local, "0.0.0.0:25566");
        assert_eq!(other[2].local, "[::]:25566");
        assert_eq!(other[3].remote, "10.144.144.1:25566");
    }

    #[test]
    fn retry_succeeds_on_second_attempt() {
        let calls = std::cell::Cell::new(0);
        let result = retry(3, || {
            calls.set(calls.get() + 1);
            if calls.get() < 2 {
                Err("fail".into())
            } else {
                Ok(())
            }
        });
        assert!(result.is_ok());
        assert_eq!(calls.get(), 2);
    }

    #[test]
    fn retry_exhausts_attempts() {
        let calls = std::cell::Cell::new(0);
        let result: Result<(), String> = retry(3, || {
            calls.set(calls.get() + 1);
            Err("fail".into())
        });
        assert!(result.is_err());
        assert_eq!(calls.get(), 3);
    }
}
