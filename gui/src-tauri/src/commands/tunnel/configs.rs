use tauri::{AppHandle};

use crate::{
    commands::tunnel::{
        metadata::{get_all_tunnels, remove_metadata_from_store, TunnelMetadata},
        stop_tunnel,
    },
    TunnelState,
};

#[tauri::command]
pub async fn get_configs(app: AppHandle) -> Result<Vec<TunnelMetadata>, String> {
    let tunnels = get_all_tunnels(&app)?;
    Ok(tunnels)
}

#[tauri::command]
pub async fn remove_config(
    app: AppHandle,
    tunnel_state: tauri::State<'_, TunnelState>,
    config: TunnelMetadata,
) -> Result<(), String> {
    let is_active = {
        let active_tunnel = tunnel_state.active_tunnel.lock().unwrap();
        active_tunnel.as_ref() == Some(&config.public_ip.to_string())
    };

    if is_active {
        stop_tunnel(app.clone(), tunnel_state).await?;
    }

    remove_metadata_from_store(&app, config.public_ip.to_string())
}
