mod commands;
mod sidecar_bridge;

use dashmap::DashMap;
use serde::Serialize;
use std::sync::Mutex;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    Manager,
};
use tauri_plugin_dialog;
use tauri_plugin_store::StoreExt;
use vpn_lib::wireguard::server::TunnelMode;

use crate::commands::{
    pinger::PingHandle,
    state::{start_monitoring, sync_tunnel_state},
    tunnel::{self, RedirectionState},
};

#[derive(Default)]
struct AppCache {
    icons: DashMap<String, String>,
}

pub struct TunnelState {
    pub active_tunnel: Mutex<Option<String>>,
    pub mode: Mutex<TunnelMode>,
}

impl Default for TunnelState {
    fn default() -> Self {
        Self {
            active_tunnel: Mutex::new(None),
            mode: Mutex::new(TunnelMode::Full),
        }
    }
}

#[derive(Clone, Serialize)]
pub struct TunnelPayload {
    pub name: Option<String>,
    pub is_active: bool,
    pub mode: TunnelMode,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let _ = std::env::set_current_dir(exe_dir);
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(AppCache::default())
        .manage(TunnelState::default())
        .manage(PingHandle(Mutex::new(None)))
        .manage(RedirectionState::default())
        .setup(|app| {
            #[cfg(desktop)]
            let _ = app.handle().plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}));

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            let resource_path = app
                .path()
                .resolve(".env", tauri::path::BaseDirectory::Resource)?;
            dotenvy::from_path(resource_path).ok();

            let data_dir = app
                .path()
                .app_local_data_dir()
                .expect("failed to get data dir");
            std::fs::create_dir_all(&data_dir).ok();

            let store_path = data_dir.join("tunnels.json");
            let _store = app.store(store_path)?;

            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                sync_tunnel_state(handle.clone()).await;
                start_monitoring(handle);
            });

            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
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
                .on_tray_icon_event(|tray, event| match event {
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
                })
                .build(app)?;

            let args: Vec<String> = std::env::args().collect();
            let should_minimize = args.contains(&"--minimized".to_string());

            if let Some(window) = app.get_webview_window("main") {
                if should_minimize {
                    window.hide().unwrap();
                } else {
                    window.show().unwrap();
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::tunnel::setup_server,
            commands::tunnel::toggle_vpn,
            commands::tunnel::start_tunnel,
            commands::tunnel::stop_tunnel,
            commands::tunnel::quick_connect,
            commands::tunnel::is_tunnel_active,
            commands::state::get_current_tunnel_status,
            commands::geo::get_geo_info,
            commands::pinger::start_ping_loop,
            commands::pinger::stop_ping_loop,
            commands::tunnel::configs::get_configs,
            commands::tunnel::configs::remove_config,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|app_handle, event| {
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let handle = app_handle.clone();

                tauri::async_runtime::block_on(async move {
                    let tunnel_state = handle.state::<TunnelState>();
                    let redirection_state = handle.state::<RedirectionState>();

                    let _ = commands::tunnel::stop_tunnel(
                        handle.clone(),
                        tunnel_state,
                    )
                    .await;
                });
            }
        });
}
