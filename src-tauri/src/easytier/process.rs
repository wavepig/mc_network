use crate::logging::LogBuffer;
use std::ffi::OsStr;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::thread;

pub struct EasyTierProcess {
    child: Child,
    pub rpc_port: u16,
}

impl EasyTierProcess {
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

impl Drop for EasyTierProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

pub fn spawn(
    program: impl AsRef<OsStr>,
    args: &[impl AsRef<OsStr>],
    rpc_port: u16,
    log: Arc<LogBuffer>,
) -> std::io::Result<EasyTierProcess> {
    let mut command = Command::new(program);
    command
        .args(args)
        .current_dir(std::env::temp_dir())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    let mut child = command.spawn()?;
    pump(child.stdout.take(), Arc::clone(&log));
    pump(child.stderr.take(), Arc::clone(&log));
    Ok(EasyTierProcess { child, rpc_port })
}

fn pump(stream: Option<impl std::io::Read + Send + 'static>, log: Arc<LogBuffer>) {
    if let Some(stream) = stream {
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                log.push(line);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::LogBuffer;
    use std::ffi::OsString;
    use std::sync::Arc;
    use std::time::Duration;

    #[cfg(windows)]
    fn sleeper() -> (String, Vec<OsString>) {
        ("ping".into(), vec!["-n".into(), "30".into(), "127.0.0.1".into()])
    }
    #[cfg(not(windows))]
    fn sleeper() -> (String, Vec<OsString>) {
        ("sleep".into(), vec!["30".into()])
    }

    #[cfg(windows)]
    fn echoer() -> (String, Vec<OsString>) {
        ("cmd".into(), vec!["/C".into(), "echo hello-log".into()])
    }
    #[cfg(not(windows))]
    fn echoer() -> (String, Vec<OsString>) {
        ("sh".into(), vec!["-c".into(), "echo hello-log".into()])
    }

    #[test]
    fn spawn_starts_alive_and_kill_stops() {
        let (prog, args) = sleeper();
        let log = Arc::new(LogBuffer::new(50));
        let mut p = spawn(&prog, &args, 0, Arc::clone(&log)).unwrap();
        assert!(p.is_alive());
        p.kill();
        assert!(!p.is_alive());
    }

    #[test]
    fn output_is_pumped_into_log() {
        let (prog, args) = echoer();
        let log = Arc::new(LogBuffer::new(50));
        let mut p = spawn(&prog, &args, 0, Arc::clone(&log)).unwrap();
        for _ in 0..40 {
            if log.tail(10).iter().any(|l| l.contains("hello-log")) {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = p.kill();
        panic!("log never captured child output: {:?}", log.tail(10));
    }
}
