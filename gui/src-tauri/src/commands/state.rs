use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tokio::time::sleep;

use crate::{
    TunnelPayload, TunnelState, commands::tunnel::{get_configs, is_tunnel_active, metadata::get_all_tunnels}
};

pub fn start_monitoring(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let name_to_check = {
                let state = app.state::<TunnelState>();
                let active_lock = state.active_tunnel.lock().unwrap();
                active_lock.clone()
            };

            if let Some(name) = name_to_check {
                if !is_tunnel_active(name) {
                    let state = app.state::<TunnelState>();
                    let mut active_lock = state.active_tunnel.lock().unwrap();
                    *active_lock = None;

                    app.emit(
                        "tunnel-status",
                        TunnelPayload {
                            name: None,
                            is_active: false,
                        },
                    )
                    .unwrap();
                }
            }

            sleep(Duration::from_secs(2)).await;
        }
    });
}

pub async fn sync_tunnel_state(app: AppHandle) {
    let state = app.state::<TunnelState>();

    if let Ok(tunnels) = get_all_tunnels(&app) {
        for tunnel in tunnels {
            if is_tunnel_active(tunnel.name.clone()) {
                let mut active_lock = state.active_tunnel.lock().unwrap();
                *active_lock = Some(tunnel.name.clone());

                println!("Found active tunnel: {}", tunnel.name);

                app.emit(
                    "tunnel-status",
                    TunnelPayload {
                        name: Some(tunnel.name),
                        is_active: true,
                    },
                )
                .unwrap();

                break;
            }
        }
    }
}

#[tauri::command]
pub fn get_current_tunnel_status(state: tauri::State<'_, TunnelState>) -> TunnelPayload {
    let lock = state.active_tunnel.lock().unwrap();
    TunnelPayload {
        is_active: lock.is_some(),
        name: lock.clone(),
    }
}
