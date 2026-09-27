use crate::app_state::{AppSnapshot, AppState};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub type EmitFn = Arc<dyn Fn(&str, &AppSnapshot) + Send + Sync>;

pub fn watch(state: Arc<AppState>, flow: u64, emit: EmitFn, stop_flag: Arc<AtomicBool>) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(2));
        if stop_flag.load(Ordering::SeqCst) {
            return;
        }
        if !state.is_flow_current(flow) {
            return;
        }
        if !crate::instance_is_alive() {
            crate::take_fake_server();
            crate::clear_instance();
            state.finish_error(flow, "easytier-core 进程意外退出".into());
            emit("network-status", &state.snapshot());
            return;
        }
    });
}
