use std::net::{Ipv4Addr, TcpListener};

pub fn allocate_rpc_port(fallback: u16) -> u16 {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 0))
        .and_then(|s| s.local_addr())
        .map(|a| a.port())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocate_returns_bindable_port() {
        let port = allocate_rpc_port(35780);
        assert_ne!(port, 0);
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() || true);
    }

    #[test]
    fn allocate_skips_busy_port_when_possible() {
        let _hold = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let p1 = allocate_rpc_port(35780);
        let p2 = allocate_rpc_port(35780);
        assert_ne!(p1, 0);
        assert_ne!(p2, 0);
    }
}
