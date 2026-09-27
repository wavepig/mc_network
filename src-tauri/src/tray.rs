use crate::app_state::{AppSnapshot, NetworkMode};
use crate::commands::AppHandleState;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Wry,
};

const TRAY_ID: &str = "main-tray";

struct TrayMenu {
    status: MenuItem<Wry>,
    stop: MenuItem<Wry>,
}

static TRAY_MENU: Mutex<Option<TrayMenu>> = Mutex::new(None);

/// 构建系统托盘：图标、菜单、点击行为。仅 Rust 侧实现。
pub fn init(app: &tauri::App) -> tauri::Result<()> {
    let status = MenuItem::with_id(app, "status", "状态：空闲", false, None::<&str>)?;
    let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, "stop", "停止网络", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let sep_top = PredefinedMenuItem::separator(app)?;
    let sep_bottom = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[&status, &sep_top, &show, &stop, &sep_bottom, &quit],
    )?;

    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("mc-network · 空闲")
        .menu(&menu)
        .menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "stop" => stop_network(app),
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| match event {
            TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } => show_window(tray.app_handle()),
            _ => {}
        });
    let icon = tauri::image::Image::from_bytes(include_bytes!("../icons/tray.png"))?;
    builder = builder.icon(icon);
    builder.build(app)?;

    *TRAY_MENU.lock().unwrap() = Some(TrayMenu { status, stop });
    refresh(app.handle(), &crate::runtime().state.snapshot());
    Ok(())
}

/// 网络状态变化后刷新托盘：tooltip 与菜单项。
pub fn refresh(app: &AppHandle, snap: &AppSnapshot) {
    let label = mode_label(&snap.mode);
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(format!("mc-network · {label}")));
    }
    if let Some(items) = TRAY_MENU.lock().unwrap().as_ref() {
        let _ = items.status.set_text(format!("状态：{label}"));
        let _ = items.stop.set_enabled(!matches!(snap.mode, NetworkMode::Idle));
    }
}

fn mode_label(mode: &NetworkMode) -> &'static str {
    match mode {
        NetworkMode::Idle => "空闲",
        NetworkMode::Scanning => "嗅探中",
        NetworkMode::ServerRunning => "服务端运行中",
        NetworkMode::ClientConnecting => "客户端连接中",
        NetworkMode::ClientRunning => "客户端运行中",
        NetworkMode::Error { .. } => "异常",
    }
}

fn show_window(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.unminimize();
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// 与 commands::stop_network 同一条停止链路，并把状态推给主窗口。
fn stop_network(app: &AppHandle) {
    let state = app.state::<AppHandleState>();
    let snap = crate::service::shutdown::stop(&state.0, &crate::runtime().stop_flag);
    let _ = app.emit("network-status", &snap);
    refresh(app, &snap);
}
