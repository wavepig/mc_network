use crate::app_state::{AppSnapshot, AppState};
use crate::service::{client, server, shutdown};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};

pub struct AppHandleState(pub Arc<AppState>);

fn emit_snapshot(app: &AppHandle, topic: &str, snap: &AppSnapshot) {
    let _ = app.emit(topic, snap);
    crate::tray::refresh(app, snap);
}

fn emit_log(app: &AppHandle, line: String) {
    let _ = app.emit("log-line", line);
}

fn validate_nodes(public_nodes: Vec<String>) -> Result<Vec<String>, String> {
    if public_nodes.is_empty() {
        return Err("至少需要一个公网节点".into());
    }
    Ok(public_nodes)
}

#[tauri::command]
pub fn create_server_network(
    app: AppHandle,
    state: State<'_, AppHandleState>,
    name: String,
    secret: String,
    public_nodes: Vec<String>,
) -> Result<AppSnapshot, String> {
    let nodes = validate_nodes(public_nodes)?;
    let state_arc: Arc<AppState> = Arc::clone(&state.0);
    let emit_status_app = app.clone();
    let emit_scan_app = app.clone();
    let log_app = app.clone();
    server::create_server(
        &state_arc,
        move |topic, snap| emit_snapshot(&emit_status_app, topic, snap),
        move |found, port| {
            let _ = emit_scan_app.emit("mc-scan", serde_json::json!({ "hit": found, "port": port }));
        },
        move |line| emit_log(&log_app, line),
        name,
        secret,
        nodes,
        stop_flag(),
    )
}

#[tauri::command]
pub fn create_client_network(
    app: AppHandle,
    state: State<'_, AppHandleState>,
    name: String,
    secret: String,
    public_nodes: Vec<String>,
    mc_port: u16,
) -> Result<AppSnapshot, String> {
    let nodes = validate_nodes(public_nodes)?;
    let state_arc: Arc<AppState> = Arc::clone(&state.0);
    let emit_app = app.clone();
    let log_app = app.clone();
    client::create_client(
        &state_arc,
        move |topic, snap| emit_snapshot(&emit_app, topic, snap),
        move |line| emit_log(&log_app, line),
        name,
        secret,
        nodes,
        mc_port,
        stop_flag(),
    )
}

#[tauri::command]
pub fn stop_network(app: AppHandle, state: State<'_, AppHandleState>) -> AppSnapshot {
    let snap = shutdown::stop(&state.0, &stop_flag());
    crate::tray::refresh(&app, &snap);
    snap
}

#[tauri::command]
pub fn get_status(state: State<'_, AppHandleState>) -> AppSnapshot {
    state.0.snapshot()
}

#[tauri::command]
pub fn get_logs() -> Vec<String> {
    crate::log_buffer().tail(100)
}

fn stop_flag() -> Arc<AtomicBool> {
    Arc::clone(&crate::runtime().stop_flag)
}
