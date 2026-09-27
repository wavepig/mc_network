pub mod fake_server;
pub mod scanner;

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

/// 测试互斥：fake_server / scanner 的用例都要抢 4445 端口，并行跑会互相收到对方的包。
#[cfg(test)]
pub(crate) static TEST_4445: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub struct ScanHit {
    pub port: u16,
    pub motd: String,
}

pub fn parse_ad_packet(data: &[u8]) -> Option<ScanHit> {
    let text = String::from_utf8_lossy(data);
    let motd_start = text.find("[MOTD]")? + 6;
    let motd_end = text.find("[/MOTD]")?;
    if motd_end <= motd_start {
        return None;
    }
    let motd = text[motd_start..motd_end].to_string();
    let ad_start = text.find("[AD]")? + 4;
    let ad_end = text.find("[/AD]")?;
    if ad_end <= ad_start {
        return None;
    }
    let port = text[ad_start..ad_end].parse().ok()?;
    Some(ScanHit { port, motd })
}

pub fn build_ad_packet(motd: &str, port: u16) -> String {
    format!("[MOTD]{motd}[/MOTD][AD]{port}[/AD]")
}

pub fn listen_addresses() -> Vec<IpAddr> {
    let mut addresses: Vec<IpAddr> = vec![];
    if let Ok(networks) = local_ip_address::list_afinet_netifas() {
        for (_, address) in networks {
            match address {
                IpAddr::V4(ip) => {
                    let o = ip.octets();
                    if !(o[0] == 10 && o[1] == 144 && o[2] == 144)
                        && ip != Ipv4Addr::LOCALHOST
                        && ip != Ipv4Addr::UNSPECIFIED
                    {
                        addresses.push(address);
                    }
                }
                IpAddr::V6(ip) => {
                    if ip != Ipv6Addr::LOCALHOST && ip != Ipv6Addr::UNSPECIFIED {
                        addresses.push(address);
                    }
                }
            }
        }
    }
    addresses.push(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
    addresses.push(IpAddr::V6(Ipv6Addr::UNSPECIFIED));
    addresses.sort_by(|a, b| b.cmp(a));
    addresses.dedup();
    addresses
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_packet() {
        let hit = parse_ad_packet(b"[MOTD]scaffolding-mc-A1B2-C3D4[/MOTD][AD]25565[/AD]").unwrap();
        assert_eq!(hit.port, 25565);
        assert_eq!(hit.motd, "scaffolding-mc-A1B2-C3D4");
    }

    #[test]
    fn parse_rejects_empty_motd() {
        assert!(parse_ad_packet(b"[MOTD][/MOTD][AD]25565[/AD]").is_none());
    }

    #[test]
    fn parse_rejects_bad_port() {
        assert!(parse_ad_packet(b"[MOTD]x[/MOTD][AD]abc[/AD]").is_none());
    }

    #[test]
    fn parse_rejects_missing_tags() {
        assert!(parse_ad_packet(b"[MOTD]x[/MOTD]").is_none());
    }

    #[test]
    fn parse_rejects_reversed_ad_tags() {
        assert!(parse_ad_packet(b"[/AD][MOTD]ok[/MOTD][AD]9[/AD]").is_none());
        assert!(parse_ad_packet(b"x[/AD][AD]5[MOTD]m[/MOTD]").is_none());
    }

    #[test]
    fn build_roundtrips() {
        let pkt = build_ad_packet("name", 25565);
        assert_eq!(pkt, "[MOTD]name[/MOTD][AD]25565[/AD]");
        let hit = parse_ad_packet(pkt.as_bytes()).unwrap();
        assert_eq!(hit.port, 25565);
    }

    #[test]
    fn listen_addresses_excludes_easytier_subnet() {
        let addrs = listen_addresses();
        assert!(addrs.iter().any(|a| a.is_unspecified()));
        for a in &addrs {
            if let std::net::IpAddr::V4(ip) = a {
                let o = ip.octets();
                assert!(!(o[0] == 10 && o[1] == 144 && o[2] == 144), "leaked easytier subnet {a}");
            }
        }
    }
}
