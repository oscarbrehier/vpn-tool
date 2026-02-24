use std::{
    net::Ipv4Addr,
    path::{Path, PathBuf},
    process::Command,
};

use tauri::{AppHandle, Manager};
use vpn_lib::{
    self,
    ssh::{connect_ssh, harden_ssh},
    validate_key_file,
    wireguard::{client::list_local_configs, server::setup_wireguard},
};

use crate::TunnelState;

#[tauri::command]
pub async fn setup_server(server_ip: String, user: String, key_file: String) -> Result<(), String> {
    println!("{}|{}|{}", server_ip, user, key_file);

    let ip: Ipv4Addr = server_ip
        .parse()
        .map_err(|_| "Invalid IP address format".to_string())?;

    let key_path = PathBuf::from(&key_file);

    validate_key_file(&key_path).map_err(|e| e.to_string())?;

    let session = connect_ssh(ip, user, key_path)
        .await
        .map_err(|e| e.to_string())?;

    setup_wireguard(&session, ip, "eth0".into())
        .await
        .map_err(|e| e.to_string())?;
    harden_ssh(&session).await.map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_vpn(connect: bool) -> Result<bool, String> {
    let action = if connect { "up" } else { "down" };
    let interface = "wg0";

    let mut cmd;

    #[cfg(target_os = "windows")]
    {
        cmd = Command::new("wg-quick");
        cmd.arg(action).arg(interface);
    }

    #[cfg(target_os = "linux")]
    {
        cmd = Command::new("pkexec");
        cmd.arg("wg-quick").arg(action).arg(interface);
    }

    #[cfg(target_os = "macos")]
    {
        cmd = Command::new("osascript");
        cmd.arg("-e").arg(format!(
            "do shell script \"wg-quick {} {}\" with administrator privileges",
            action, interface
        ));
    }

    let status = cmd.status().map_err(|e| e.to_string())?;

    Ok(status.success())
}

#[tauri::command]
pub async fn get_configs() -> Result<Vec<String>, String> {
    let conf_dir = Path::new("conf");
    list_local_configs(conf_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_tunnel_active(name: String) -> bool {
    #[cfg(target_os = "windows")]
    {
        let service_name = format!("WireGuardTunnel${}", name);
        let output = std::process::Command::new("sc")
            .args(["query", &service_name])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout);
                stdout.contains("RUNNING")
            }
            Err(_) => false,
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = std::process::Command::new("wg")
            .args(["show", &name])
            .output();
        output.map(|o| o.status.success()).unwrap_or(false)
    }
}

#[tauri::command]
pub async fn start_tunnel(
    app: AppHandle,
    state: tauri::State<'_, TunnelState>,
    conf_name: String,
) -> Result<(), String> {
    let mut conf_path = app
        .path()
        .resource_dir()
        .map_err(|e| e.to_string())?
        .join("conf")
        .join(&conf_name);

    if !conf_path.exists() && cfg!(debug_assertions) {
        conf_path = app
            .path()
            .app_config_dir()
            .map_err(|e| e.to_string())?
            .parent()
            .unwrap()
            .join("conf")
            .join(&conf_name);
    }

    if !conf_path.exists() {
        return Err(format!("Configuration file not found at {:?}", conf_path));
    }

    vpn_lib::wireguard::client::start_tunnel(&conf_path).map_err(|e| e.to_string())?;

    let tunnel_name = conf_name.replace(".conf", "");
    let mut active_lock = state.active_tunnel.lock().unwrap();
    *active_lock = Some(tunnel_name);

    Ok(())
}

#[tauri::command]
pub async fn stop_tunnel(app: AppHandle) -> Result<(), String> {
    let tunnel_name = {
        let state = app.state::<TunnelState>();
        let active_lock = state.active_tunnel.lock().unwrap();
        active_lock.clone()
    };

    let Some(name) = tunnel_name else {
        return Err("No active tunnel found in state".to_string());
    };

    Ok(())
}
