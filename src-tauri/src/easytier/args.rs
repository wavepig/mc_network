use std::ffi::OsString;

pub const SERVER_IPV4: &str = "10.144.144.1";
pub const FALLBACK_RPC_SERVER: u16 = 35780;
pub const FALLBACK_RPC_CLIENT: u16 = 35781;

const COMMON_FLAGS: &[&str] = &[
    "--no-tun",
    "--compression=zstd",
    "--multi-thread",
    "--latency-first",
    "--enable-kcp-proxy",
    "-l",
    "udp://0.0.0.0:0",
    "-l",
    "tcp://0.0.0.0:0",
];

pub struct ServerArgs {
    pub name: String,
    pub secret: String,
    pub public_nodes: Vec<String>,
    pub hostname: String,
    pub tcp_whitelist: u16,
    pub udp_whitelist: u16,
    pub rpc_port: u16,
}

impl ServerArgs {
    pub fn to_command_line(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        push_pair(&mut args, "--network-name", &self.name);
        push_pair(&mut args, "--network-secret", &self.secret);
        for node in &self.public_nodes {
            push_pair(&mut args, "-p", node);
        }
        args.extend(COMMON_FLAGS.iter().map(OsString::from));
        args.push("--p2p-only".into());
        push_pair(&mut args, "--hostname", &self.hostname);
        push_pair(&mut args, "--ipv4", SERVER_IPV4);
        args.push(format!("--tcp-whitelist={}", self.tcp_whitelist).into());
        args.push(format!("--udp-whitelist={}", self.udp_whitelist).into());
        push_pair(&mut args, "-r", &self.rpc_port.to_string());
        args
    }
}

pub struct ClientArgs {
    pub name: String,
    pub secret: String,
    pub public_nodes: Vec<String>,
    pub rpc_port: u16,
}

impl ClientArgs {
    pub fn to_command_line(&self) -> Vec<OsString> {
        let mut args: Vec<OsString> = Vec::new();
        push_pair(&mut args, "--network-name", &self.name);
        push_pair(&mut args, "--network-secret", &self.secret);
        for node in &self.public_nodes {
            push_pair(&mut args, "-p", node);
        }
        args.extend(COMMON_FLAGS.iter().map(OsString::from));
        args.push("-d".into());
        args.push("--tcp-whitelist=0".into());
        args.push("--udp-whitelist=0".into());
        push_pair(&mut args, "-r", &self.rpc_port.to_string());
        args
    }
}

fn push_pair(args: &mut Vec<OsString>, key: &str, value: &str) {
    args.push(key.into());
    args.push(value.into());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(args: &[std::ffi::OsString]) -> String {
        args.iter().map(|s| s.to_string_lossy().into_owned()).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn server_command_line_matches_requirement_case() {
        let args = ServerArgs {
            name: "scaffolding-mc-A1B2-C3D4".into(),
            secret: "E5F6-G7H8".into(),
            public_nodes: vec![
                "tcp://public.easytier.top:11010".into(),
                "tcp://public2.easytier.cn:54321".into(),
                "https://etnode.zkitefly.eu.org/node1".into(),
                "https://etnode.zkitefly.eu.org/node2".into(),
            ],
            hostname: "mc-network-550e8400-e29b-41d4-a716-446655440000".into(),
            tcp_whitelist: 25565,
            udp_whitelist: 25565,
            rpc_port: 35780,
        };
        assert_eq!(
            joined(&args.to_command_line()),
            "--network-name scaffolding-mc-A1B2-C3D4 --network-secret E5F6-G7H8 \
-p tcp://public.easytier.top:11010 -p tcp://public2.easytier.cn:54321 \
-p https://etnode.zkitefly.eu.org/node1 -p https://etnode.zkitefly.eu.org/node2 \
--no-tun --compression=zstd --multi-thread --latency-first --enable-kcp-proxy \
-l udp://0.0.0.0:0 -l tcp://0.0.0.0:0 --p2p-only \
--hostname mc-network-550e8400-e29b-41d4-a716-446655440000 --ipv4 10.144.144.1 \
--tcp-whitelist=25565 --udp-whitelist=25565 -r 35780"
        );
        let vec = args.to_command_line();
        assert_eq!(vec.len(), 30);
        assert_eq!(vec[0], std::ffi::OsString::from("--network-name"));
        assert_eq!(vec[17], std::ffi::OsString::from("-l"));
        assert_eq!(vec[18], std::ffi::OsString::from("udp://0.0.0.0:0"));
        assert_eq!(vec[26], std::ffi::OsString::from("--tcp-whitelist=25565"));
        assert_eq!(vec[28], std::ffi::OsString::from("-r"));
    }

    #[test]
    fn client_command_line_matches_requirement_case() {
        let args = ClientArgs {
            name: "scaffolding-mc-A1B2-C3D4".into(),
            secret: "E5F6-G7H8".into(),
            public_nodes: vec![
                "https://etnode.zkitefly.eu.org/node1".into(),
                "https://etnode.zkitefly.eu.org/node2".into(),
            ],
            rpc_port: 35781,
        };
        assert_eq!(
            joined(&args.to_command_line()),
            "--network-name scaffolding-mc-A1B2-C3D4 --network-secret E5F6-G7H8 \
-p https://etnode.zkitefly.eu.org/node1 -p https://etnode.zkitefly.eu.org/node2 \
--no-tun --compression=zstd --multi-thread --latency-first --enable-kcp-proxy \
-l udp://0.0.0.0:0 -l tcp://0.0.0.0:0 -d --tcp-whitelist=0 --udp-whitelist=0 -r 35781"
        );
        let vec = args.to_command_line();
        assert_eq!(vec.len(), 22);
        assert_eq!(vec[13], std::ffi::OsString::from("-l"));
        assert_eq!(vec[14], std::ffi::OsString::from("udp://0.0.0.0:0"));
        assert_eq!(vec[18], std::ffi::OsString::from("--tcp-whitelist=0"));
        assert_eq!(vec[20], std::ffi::OsString::from("-r"));
    }
}
