mod commands;

use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_dialog;

use crate::commands::{
    pinger::PingHandle,
    state::{start_monitoring, sync_tunnel_state},
};

#[derive(Default)]
pub struct TunnelState {
    pub active_tunnel: Mutex<Option<String>>,
}

#[derive(Clone, Serialize)]
pub struct TunnelPayload {
    pub name: Option<String>,
    pub is_active: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    tauri::Builder::default()
        .manage(TunnelState::default())
        .manage(PingHandle(Mutex::new(None)))
        .setup(|app| {
            let handle = app.handle().clone();

            tauri::async_runtime::block_on(async {
                sync_tunnel_state(handle.clone()).await;
            });

            start_monitoring(handle);

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => app.exit(0),
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    match event {
                        TrayIconEvent::Click {
                            button: tauri::tray::MouseButton::Left,
                            ..
                        } => {
                            let app = tray.app_handle();
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::tunnel::setup_server,
            commands::tunnel::toggle_vpn,
            commands::tunnel::get_configs,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::quick_connect,
            commands::tunnel::is_tunnel_active,
            commands::state::get_current_tunnel_status,
            commands::geo::get_geo_info,
            commands::pinger::start_ping_loop,
            commands::pinger::stop_ping_loop,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
