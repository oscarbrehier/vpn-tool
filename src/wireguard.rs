use std::{io::Write, path::Path};
use anyhow::Context;
use base64::{Engine, engine::general_purpose};
use ssh2::{Session};
use x25519_dalek::{PublicKey, StaticSecret};
use crate::ssh::run_remote_cmd;
use rand_core::OsRng;

fn generate_keys() -> (String, String) {

	let secret = StaticSecret::random_from_rng(OsRng);
	let public = PublicKey::from(&secret);

	let priv_b64 = general_purpose::STANDARD.encode(secret.to_bytes());
	let pub_b64 = general_purpose::STANDARD.encode(public.to_bytes());

	(priv_b64, pub_b64)

}

fn build_server_config(
    server_private_key: &str,
    client_public_key: &str,
    interface: &str,
) -> String {
    format!(
        r#"[Interface]
Address = 10.0.0.1/24
ListenPort = 51820
PrivateKey = {server_private_key}

PostUp = iptables -A FORWARD -i %i -j ACCEPT; iptables -t nat -A POSTROUTING -o {interface} -j MASQUERADE
PostDown = iptables -D FORWARD -i %i -j ACCEPT; iptables -t nat -D POSTROUTING -o {interface} -j MASQUERADE

[Peer]
PublicKey = {client_public_key}
AllowedIPs = 10.0.0.2/32
        "#
    )
}

fn upload_config(session: &Session, config_content: &str) -> anyhow::Result<()> {
    let remote_path = Path::new("/etc/wireguard/wg0.conf");

    let mut remote_file = session
        .scp_send(remote_path, 0o600, config_content.len() as u64, None)
        .context("Failed to initiate SCP transfer")?;

    remote_file.write_all(config_content.as_bytes())?;

    remote_file.send_eof()?;
    remote_file.wait_eof()?;
    remote_file.close()?;
    remote_file.wait_close()?;

    Ok(())
}

pub fn setup_wireguard(session: &Session, interface: &str) -> anyhow::Result<()> {
    let (_, status) = run_remote_cmd(session, "which wg")?;

    if status != 0 {

        let (_, install_status) =
            run_remote_cmd(session, "sudo apt update && sudo apt install -y wireguard")?;
        if install_status != 0 {
            anyhow::bail!("Wireguard installation failed");
        }
    }
	let (server_priv, server_pub) = generate_keys();
	let (client_priv, client_pub) = generate_keys();

    let server_config = build_server_config(&server_priv, &client_pub, interface);
    upload_config(session, &server_config)?;

    run_remote_cmd(session, "sudo wg-quick up wg0")?;

    println!("Server is up! Use these for your client config:");
    println!("Client Private: {}", client_priv);
    println!("Server Public: {}", server_pub);

    Ok(())
}