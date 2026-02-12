use crate::ssh::run_remote_cmd;
use anyhow::Context;
use base64::{Engine, engine::general_purpose};
use rand_core::OsRng;
use ssh2::Session;
use std::{io::Write, net::IpAddr, path::Path};
use x25519_dalek::{PublicKey, StaticSecret};

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

fn build_client_config(client_priv: &str, server_pub: &str, vps_ip: &str) -> String {
    format!(
        r#"[Interface]
PrivateKey = {client_priv}
Address = 10.0.0.2/24
DNS = 1.1.1.1

[Peer]
PublicKey = {server_pub}
Endpoint = {vps_ip}:51820
AllowedIPs = 0.0.0.0/0
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

pub fn setup_wireguard(session: &Session, vps_ip: &IpAddr, interface: &str) -> anyhow::Result<()> {
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

    let client_config = build_client_config(&client_priv, &server_pub, &vps_ip.to_string());

    let filename = format!("wg_{}.conf", vps_ip);
    std::fs::write(&filename, &client_config)?;

    println!("Success! Configuration saved to: {}", filename);
    println!("Import this file into Wireguard Client to connect");

    Ok(())
}
