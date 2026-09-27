use crate::app_state::{AppSnapshot, AppState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub fn stop(state: &Arc<AppState>, stop_flag: &Arc<AtomicBool>) -> AppSnapshot {
    stop_flag.store(true, Ordering::SeqCst);
    crate::take_fake_server();
    crate::kill_instance();
    stop_flag.store(false, Ordering::SeqCst);
    state.stop()
}
