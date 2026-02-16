use std::{net::Ipv4Addr, path::{PathBuf}};

use vpn_lib::{self, connect_ssh, ssh::harden_ssh, validate_key_file, wireguard::server::setup_wireguard};

#[tauri::command]
pub async fn setup_server(server_ip: String, key_file: String, user: String) -> Result<(), String> {
	
	let ip: Ipv4Addr = server_ip.parse().map_err(|_| "Invalid IP address format".to_string())?;

	let key_path = PathBuf::from(&key_file);

	validate_key_file(&key_path)
		.map_err(|e| e.to_string())?;

	let session = connect_ssh(ip, user, key_path).await.map_err(|e| e.to_string())?;

	setup_wireguard(&session, ip, "initial_client".into()).map_err(|e| e.to_string())?;
	harden_ssh(&session).map_err(|e| e.to_string())?;

	Ok(())

}