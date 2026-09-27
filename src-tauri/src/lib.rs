mod app_state;
mod commands;
mod easytier;
mod logging;
mod mc;
mod ports;
mod service;
mod tray;

use easytier::process::EasyTierProcess;
use logging::LogBuffer;
use mc::fake_server::FakeServer;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, OnceLock};

pub struct Runtime {
    pub state: Arc<app_state::AppState>,
    pub log: Arc<LogBuffer>,
    pub stop_flag: Arc<AtomicBool>,
    instance: Mutex<Option<EasyTierProcess>>,
    fake_server: Mutex<Option<FakeServer>>,
}

static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime {
        state: Arc::new(app_state::AppState::new()),
        log: Arc::new(LogBuffer::new(500)),
        stop_flag: Arc::new(AtomicBool::new(false)),
        instance: Mutex::new(None),
        fake_server: Mutex::new(None),
    })
}

pub fn log_buffer() -> Arc<LogBuffer> {
    Arc::clone(&runtime().log)
}

pub fn set_instance(p: EasyTierProcess) {
    *runtime().instance.lock().unwrap() = Some(p);
}

pub fn kill_instance() {
    if let Some(mut p) = runtime().instance.lock().unwrap().take() {
        p.kill();
    }
}

pub fn instance_is_alive() -> bool {
    runtime()
        .instance
        .lock()
        .unwrap()
        .as_mut()
        .map(|p| p.is_alive())
        .unwrap_or(true)
}

pub fn clear_instance() {
    runtime().instance.lock().unwrap().take();
}

pub fn set_fake_server_slot(slot: Arc<Mutex<Option<FakeServer>>>) {
    let server = slot.lock().unwrap().take();
    *runtime().fake_server.lock().unwrap() = server;
}

pub fn take_fake_server() {
    if let Some(server) = runtime().fake_server.lock().unwrap().take() {
        server.stop();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(commands::AppHandleState(runtime().state.clone()))
        .invoke_handler(tauri::generate_handler![
            commands::create_server_network,
            commands::create_client_network,
            commands::stop_network,
            commands::get_status,
            commands::get_logs,
        ])
        .setup(|app| {
            let window = tauri::WebviewWindowBuilder::new(
                app,
                "main",
                tauri::WebviewUrl::App("index.html".into()),
            )
            .title("mc-network")
            .inner_size(1295.0, 830.0)
            .min_inner_size(720.0, 520.0)
            .center()
            .enable_clipboard_access()
            .build()?;

            // 关闭按钮：隐藏到托盘，网络继续运行；托盘菜单「退出」才真正退出
            let hide_target = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = hide_target.hide();
                }
            });

            tray::init(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let _ = app;
                take_fake_server();
                kill_instance();
                easytier::binaries::cleanup();
            }
        });
}
