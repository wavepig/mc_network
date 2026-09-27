use super::build_ad_packet;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// MC「局域网游戏」不解析公告里的地址——房间地址取自 UDP 包的源地址。
/// 因此发送 socket 固定绑定 127.0.0.1：房间始终显示为 127.0.0.1:{port}，
/// 点击后走 IPv4 回环，正好命中本地端口转发（0.0.0.0 / [::]）的监听。
const AD_PORT: u16 = 4445;

pub struct FakeServer {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl FakeServer {
    pub fn stop(mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for FakeServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
    }
}

pub fn start(network_name: String, port: u16) -> FakeServer {
    let stop = Arc::new(AtomicBool::new(false));
    let thread_stop = Arc::clone(&stop);
    let handle = thread::spawn(move || run(network_name, port, thread_stop));
    FakeServer { stop, handle: Some(handle) }
}

fn run(network_name: String, port: u16, stop: Arc<AtomicBool>) {
    let message = build_ad_packet(&network_name, port);
    let Some(socket) = open_sender() else {
        return;
    };
    // 单播到 127.0.0.1:4445：MC 的局域网监听 socket 绑定 0.0.0.0:4445，单播同样能收到。
    let target = SocketAddr::from((Ipv4Addr::LOCALHOST, AD_PORT));

    while !stop.load(Ordering::SeqCst) {
        let _ = socket.send_to(message.as_bytes(), target);
        thread::sleep(Duration::from_millis(1500));
    }
}

fn open_sender() -> Option<UdpSocket> {
    UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use socket2::{Domain, Protocol, SockAddr, Socket, Type};
    use std::net::{IpAddr, Ipv4Addr, SocketAddrV4, UdpSocket};
    use std::time::Duration;

    fn bind_listener() -> UdpSocket {
        let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP)).unwrap();
        socket.set_reuse_address(true).unwrap();
        socket.bind(&SockAddr::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 4445))).unwrap();
        socket
            .join_multicast_v4(&Ipv4Addr::new(224, 0, 2, 60), &Ipv4Addr::UNSPECIFIED)
            .unwrap();
        socket.set_read_timeout(Some(Duration::from_millis(2500))).unwrap();
        socket.into()
    }

    #[test]
    fn broadcasts_ad_packet_and_stops() {
        let _guard = crate::mc::TEST_4445.lock().unwrap_or_else(|e| e.into_inner());
        let listener = bind_listener();
        let server = start("scaffolding-mc-A1B2-C3D4".into(), 25565);

        let mut buf = [0u8; 512];
        let mut from_loopback = None;
        for _ in 0..8 {
            let (n, from) = listener.recv_from(&mut buf).expect("expected broadcast");
            if from.ip() == IpAddr::V4(Ipv4Addr::LOCALHOST) {
                from_loopback = Some(String::from_utf8_lossy(&buf[..n]).into_owned());
                break;
            }
        }
        let message = from_loopback.expect("expected announcement sourced from 127.0.0.1");
        assert_eq!(
            message,
            "[MOTD]scaffolding-mc-A1B2-C3D4[/MOTD][AD]25565[/AD]"
        );

        server.stop();
        // 最后一个周期可能还在队列里，先排空，再确认彻底安静
        listener.set_read_timeout(Some(Duration::from_millis(300))).unwrap();
        while listener.recv_from(&mut buf).is_ok() {}
        assert!(listener.recv_from(&mut buf).is_err(), "should stop broadcasting");
    }
}
