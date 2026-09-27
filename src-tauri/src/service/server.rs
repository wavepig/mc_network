use super::watchdog;
use crate::app_state::{AppSnapshot, AppState, NetworkMode};
use crate::easytier::args::{ServerArgs, FALLBACK_RPC_SERVER};
use crate::easytier::binaries;
use crate::easytier::process;
use crate::mc::scanner;
use crate::ports::allocate_rpc_port;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

pub fn generate_hostname() -> String {
    format!("mc-network-{}", Uuid::new_v4())
}

pub fn validate(name: &str, secret: &str, nodes: Vec<String>) -> Result<Vec<String>, String> {
    if name.trim().is_empty() {
        return Err("network-name 不能为空".into());
    }
    if secret.trim().is_empty() {
        return Err("network-secret 不能为空".into());
    }
    if nodes.is_empty() {
        return Err("至少需要一个公网节点".into());
    }
    Ok(nodes)
}

pub fn create_server(
    state: &Arc<AppState>,
    emit: impl Fn(&str, &AppSnapshot) + Send + Sync + 'static,
    emit_scan: impl Fn(bool, Option<u16>) + Send + Sync + 'static,
    emit_log: impl Fn(String) + Send + Sync + 'static,
    name: String,
    secret: String,
    nodes: Vec<String>,
    stop_flag: Arc<AtomicBool>,
) -> Result<AppSnapshot, String> {
    let nodes = validate(&name, &secret, nodes)?;
    let flow = state.begin(NetworkMode::Scanning)?;
    let worker_state = Arc::clone(state);
    let emit: watchdog::EmitFn = Arc::new(emit);

    thread::spawn(move || {
        let hit = scanner::scan_until_hit(Duration::from_secs(30), Arc::clone(&stop_flag), |h| {
            emit_scan(true, Some(h.port));
        });

        let Some(hit) = hit else {
            emit_scan(false, None);
            worker_state.finish_error(flow, "未发现 MC 局域网广播，请先在 MC 中“对局域网开放”后重试".into());
            emit("network-status", &worker_state.snapshot());
            return;
        };

        let hostname = generate_hostname();
        let rpc_port = allocate_rpc_port(FALLBACK_RPC_SERVER);
        let args = ServerArgs {
            name: name.clone(),
            secret: secret.clone(),
            public_nodes: nodes,
            hostname: hostname.clone(),
            tcp_whitelist: hit.port,
            udp_whitelist: hit.port,
            rpc_port,
        };

        let paths = binaries::acquire();
        let log = crate::log_buffer();
        let spawned = process::spawn(paths.core.as_path(), &args.to_command_line(), rpc_port, log);
        let Ok(mut child) = spawned else {
            worker_state.finish_error(flow, "easytier-core 启动失败".into());
            emit("network-status", &worker_state.snapshot());
            return;
        };

        if !worker_state.update(flow, |s| {
            s.mode = NetworkMode::ServerRunning;
            s.name = Some(name);
            s.secret = Some(secret);
            s.hostname = Some(hostname);
            s.mc_port = Some(hit.port);
            s.rpc_port = Some(rpc_port);
        }) {
            child.kill();
            return;
        }
        crate::set_instance(child);
        watchdog::watch(worker_state.clone(), flow, Arc::clone(&emit), Arc::clone(&stop_flag));
        emit_log(format!("server started on rpc port {rpc_port}"));
        emit("network-status", &worker_state.snapshot());
    });

    Ok(state.snapshot())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostname_carries_mc_network_prefix_and_uuid() {
        let h = generate_hostname();
        assert!(h.starts_with("mc-network-"));
        let uuid = h.trim_start_matches("mc-network-");
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().filter(|c| *c == '-').count(), 4);
    }

    #[test]
    fn validate_rejects_blank_fields() {
        assert!(validate("", "s", vec!["n".into()]).is_err());
        assert!(validate("n", "", vec!["n".into()]).is_err());
        assert!(validate("n", "s", vec![]).is_err());
        assert!(validate("n", "s", vec!["n".into()]).is_ok());
    }
}
