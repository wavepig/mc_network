use serde::Serialize;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum NetworkMode {
    Idle,
    Scanning,
    ServerRunning,
    ClientConnecting,
    ClientRunning,
    Error { message: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct ForwardStatus {
    pub proto: String,
    pub local: String,
    pub remote: String,
    pub ok: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct AppSnapshot {
    pub mode: NetworkMode,
    pub name: Option<String>,
    pub secret: Option<String>,
    pub hostname: Option<String>,
    pub mc_port: Option<u16>,
    pub rpc_port: Option<u16>,
    pub forwards: Vec<ForwardStatus>,
}

impl Default for AppSnapshot {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Idle,
            name: None,
            secret: None,
            hostname: None,
            mc_port: None,
            rpc_port: None,
            forwards: vec![],
        }
    }
}

struct Inner {
    snapshot: AppSnapshot,
    flow_id: u64,
}

pub struct AppState {
    inner: Mutex<Inner>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner { snapshot: AppSnapshot::default(), flow_id: 0 }),
        }
    }

    pub fn begin(&self, mode: NetworkMode) -> Result<u64, String> {
        let mut inner = self.inner.lock().unwrap();
        if !matches!(inner.snapshot.mode, NetworkMode::Idle) {
            return Err("已有网络在运行".into());
        }
        inner.flow_id += 1;
        inner.snapshot = AppSnapshot { mode, ..AppSnapshot::default() };
        Ok(inner.flow_id)
    }

    pub fn update(&self, flow: u64, f: impl FnOnce(&mut AppSnapshot)) -> bool {
        let mut inner = self.inner.lock().unwrap();
        if inner.flow_id != flow {
            return false;
        }
        f(&mut inner.snapshot);
        true
    }

    pub fn is_flow_current(&self, flow: u64) -> bool {
        self.inner.lock().unwrap().flow_id == flow
    }

    pub fn finish_error(&self, flow: u64, message: String) {
        self.update(flow, |s| {
            s.mode = NetworkMode::Error { message };
        });
    }

    pub fn stop(&self) -> AppSnapshot {
        let mut inner = self.inner.lock().unwrap();
        inner.flow_id += 1;
        inner.snapshot = AppSnapshot::default();
        inner.snapshot.clone()
    }

    pub fn snapshot(&self) -> AppSnapshot {
        self.inner.lock().unwrap().snapshot.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_rejects_when_not_idle() {
        let state = AppState::new();
        state.begin(NetworkMode::Scanning).unwrap();
        assert!(state.begin(NetworkMode::ClientConnecting).is_err());
    }

    #[test]
    fn stale_flow_update_is_rejected() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        state.stop();
        assert!(!state.update(flow, |s| s.mc_port = Some(25565)));
        assert_eq!(state.snapshot().mc_port, None);
    }

    #[test]
    fn current_flow_update_applies() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        assert!(state.update(flow, |s| s.mc_port = Some(25565)));
        assert_eq!(state.snapshot().mc_port, Some(25565));
    }

    #[test]
    fn stop_returns_to_idle_and_invalidates_flow() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        let snap = state.stop();
        assert!(matches!(snap.mode, NetworkMode::Idle));
        assert!(!state.update(flow, |_| {}));
    }

    #[test]
    fn error_mode_carries_message() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        state.finish_error(flow, "boom".into());
        let snap = state.snapshot();
        assert!(matches!(&snap.mode, NetworkMode::Error { message } if message == "boom"));
    }

    #[test]
    fn flow_currency_follows_begin_and_stop() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        assert!(state.is_flow_current(flow));
        state.stop();
        assert!(!state.is_flow_current(flow));
        let next = state.begin(NetworkMode::ServerRunning).unwrap();
        assert!(state.is_flow_current(next));
        assert!(!state.is_flow_current(flow));
    }

    #[test]
    fn finish_error_on_stale_flow_is_ignored() {
        let state = AppState::new();
        let flow = state.begin(NetworkMode::Scanning).unwrap();
        state.stop();
        state.finish_error(flow, "boom".into());
        assert!(matches!(state.snapshot().mode, NetworkMode::Idle));
    }
}
