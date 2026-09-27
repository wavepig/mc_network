use serde::Deserialize;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub enum Proto {
    Tcp,
    Udp,
}

impl Proto {
    fn as_str(self) -> &'static str {
        match self {
            Proto::Tcp => "tcp",
            Proto::Udp => "udp",
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PeerInfo {
    pub hostname: Option<String>,
    pub ipv4: Option<String>,
}

pub fn peer_args(rpc: u16) -> Vec<OsString> {
    vec![
        "-p".into(),
        format!("127.0.0.1:{rpc}").into(),
        "-o".into(),
        "json".into(),
        "peer".into(),
    ]
}

pub fn port_forward_args(rpc: u16, proto: Proto, local: &str, remote: &str) -> Vec<OsString> {
    vec![
        "-p".into(),
        format!("127.0.0.1:{rpc}").into(),
        "port-forward".into(),
        "add".into(),
        proto.as_str().into(),
        local.into(),
        remote.into(),
    ]
}

pub fn run(cli: &Path, args: &[OsString]) -> Result<String, String> {
    let mut command = Command::new(cli);
    command.args(args).current_dir(std::env::temp_dir());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let output = command.output().map_err(|e| e.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "easytier-cli failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn check_peer(cli: &Path, rpc: u16) -> Result<Vec<PeerInfo>, String> {
    let raw = run(cli, &peer_args(rpc))?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

pub fn add_port_forward(
    cli: &Path,
    rpc: u16,
    proto: Proto,
    local: &str,
    remote: &str,
) -> Result<(), String> {
    run(cli, &port_forward_args(rpc, proto, local, remote)).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn joined(args: &[std::ffi::OsString]) -> String {
        args.iter().map(|s| s.to_string_lossy().into_owned()).collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn peer_args_match_requirement_case() {
        assert_eq!(joined(&peer_args(35781)), "-p 127.0.0.1:35781 -o json peer");
    }

    #[test]
    fn port_forward_args_match_requirement_case() {
        assert_eq!(
            joined(&port_forward_args(35781, Proto::Tcp, "0.0.0.0:25565", "10.144.144.1:25565")),
            "-p 127.0.0.1:35781 port-forward add tcp 0.0.0.0:25565 10.144.144.1:25565"
        );
        assert_eq!(
            joined(&port_forward_args(35781, Proto::Udp, "0.0.0.0:25565", "10.144.144.1:25565")),
            "-p 127.0.0.1:35781 port-forward add udp 0.0.0.0:25565 10.144.144.1:25565"
        );
    }
}
