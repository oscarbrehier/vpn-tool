use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle};
use tauri_plugin_store::StoreExt;
use vpn_lib::wireguard::server::SetupResult;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TunnelMetadata {
    pub name: String,
    pub server_public_key: String,
    pub client_ip: Ipv4Addr,
    pub public_ip: Ipv4Addr,
}

impl From<SetupResult> for TunnelMetadata {
    fn from(result: SetupResult) -> Self {
        Self {
            name: format!("VPN-{}", result.public_ip),
            server_public_key: result.server_public_key,
            client_ip: result.client_ip,
            public_ip: result.public_ip,
        }
    }
}

pub fn save_metadata_to_store(app: &AppHandle, data: TunnelMetadata) -> Result<(), String> {
    let store = app.store("tunnels.json").map_err(|e| e.to_string())?;

    store.set(
        data.public_ip.to_string(),
        serde_json::to_value(&data).map_err(|e| e.to_string())?,
    );

    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn get_all_tunnels(app: &AppHandle) -> Result<Vec<TunnelMetadata>, String> {

    let store = app.store("tunnels.json").map_err(|e| e.to_string())?;
    let mut tunnels = Vec::new();

    for (_key, value) in store.entries() {
        match serde_json::from_value::<TunnelMetadata>(value.clone()) {
            Ok(metadata) => tunnels.push(metadata),
            Err(e) => println!("Failed to deserialize tunnel entry: {}", e),
        }
    }

    Ok(tunnels)
}
