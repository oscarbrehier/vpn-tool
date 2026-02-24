mod commands;

use serde::Serialize;
use std::sync::Mutex;
use tauri_plugin_dialog;

use crate::commands::state::{start_monitoring, sync_tunnel_state};

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
    tauri::Builder::default()
        .manage(TunnelState::default())
        .setup(|app| {
            let handle = app.handle().clone();
            
            tauri::async_runtime::block_on(async {
                sync_tunnel_state(handle.clone()).await;
            });

            start_monitoring(handle);
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::tunnel::setup_server,
            commands::tunnel::toggle_vpn,
            commands::tunnel::get_configs,
            commands::tunnel::start_tunnel,
            commands::tunnel::is_tunnel_active,
            commands::state::get_current_tunnel_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}