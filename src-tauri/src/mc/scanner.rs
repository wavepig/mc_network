use super::{listen_addresses, parse_ad_packet, ScanHit};
use socket2::{Domain, Protocol, SockAddr, Socket, Type};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{io, thread};

pub fn scan_until_hit(
    timeout: Duration,
    stop: Arc<AtomicBool>,
    on_progress: impl Fn(&ScanHit),
) -> Option<ScanHit> {
    let sockets = open_sockets();
    let deadline = Instant::now() + timeout;
    let mut buf = [0u8; 8192];

    while Instant::now() < deadline && !stop.load(Ordering::SeqCst) {
        for socket in &sockets {
            if let Ok((n, _)) = socket.recv_from(&mut buf) {
                if let Some(hit) = parse_ad_packet(&buf[..n]) {
                    on_progress(&hit);
                    return Some(hit);
                }
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    None
}

fn open_sockets() -> Vec<UdpSocket> {
    listen_addresses()
        .into_iter()
        .filter_map(|address| open_socket(address).ok())
        .collect()
}

fn open_socket(address: std::net::IpAddr) -> io::Result<UdpSocket> {
    match address {
        std::net::IpAddr::V4(ip) => {
            let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
            socket.set_reuse_address(true)?;
            socket.bind(&SockAddr::from(SocketAddrV4::new(ip, 4445)))?;
            socket.join_multicast_v4(&Ipv4Addr::from_str("224.0.2.60").unwrap(), &Ipv4Addr::UNSPECIFIED)?;
            socket.set_read_timeout(Some(Duration::from_millis(200)))?;
            Ok(socket.into())
        }
        std::net::IpAddr::V6(ip) => {
            let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP))?;
            socket.set_only_v6(true)?;
            socket.set_reuse_address(true)?;
            socket.bind(&SockAddr::from(SocketAddrV6::new(ip, 4445, 0, 0)))?;
            socket.join_multicast_v6(&Ipv6Addr::from_str("FF75:230::60").unwrap(), 0)?;
            socket.set_read_timeout(Some(Duration::from_millis(200)))?;
            Ok(socket.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mc::build_ad_packet;
    use std::net::UdpSocket;
    use std::sync::atomic::AtomicBool;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn scan_finds_broadcast_port() {
        let _guard = crate::mc::TEST_4445.lock().unwrap_or_else(|e| e.into_inner());
        let stop = Arc::new(AtomicBool::new(false));
        let sender_stop = Arc::clone(&stop);
        let sender = thread::spawn(move || {
            let sock = UdpSocket::bind(("0.0.0.0", 0)).unwrap();
            sock.set_multicast_loop_v4(true).unwrap();
            sock.set_multicast_ttl_v4(4).unwrap();
            let pkt = build_ad_packet("scaffolding-mc-A1B2-C3D4", 25565);
            while !sender_stop.load(std::sync::atomic::Ordering::SeqCst) {
                let _ = sock.send_to(pkt.as_bytes(), ("224.0.2.60", 4445));
                thread::sleep(Duration::from_millis(200));
            }
        });

        let hit = scan_until_hit(Duration::from_secs(10), Arc::clone(&stop), |_| {});
        stop.store(true, std::sync::atomic::Ordering::SeqCst);
        sender.join().unwrap();

        let hit = hit.expect("expected scan hit");
        assert_eq!(hit.port, 25565);
        assert_eq!(hit.motd, "scaffolding-mc-A1B2-C3D4");
    }

    #[test]
    fn scan_times_out_without_traffic() {
        let _guard = crate::mc::TEST_4445.lock().unwrap_or_else(|e| e.into_inner());
        let stop = Arc::new(AtomicBool::new(false));
        let hit = scan_until_hit(Duration::from_millis(800), Arc::clone(&stop), |_| {});
        assert!(hit.is_none());
    }
}
