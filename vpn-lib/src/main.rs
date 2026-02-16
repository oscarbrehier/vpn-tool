use clap::Parser;
use std::{net::{Ipv4Addr}, path::PathBuf};
use vpn_lib::{
    connect_ssh, ping_server, ssh::{harden_ssh, run_remote_cmd}, validate_key_file, wireguard::{self, peer::add_new_peer, server::{build_client_config, get_server_public_key}, state::VpnState}
};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    ip: Ipv4Addr,
    #[arg(short, long)]
    key_path: PathBuf,
    #[arg(short, long, default_value = "dev")]
    user: String,
    #[arg(long, default_value = "eth0")]
    interface: String,
    #[arg(long)]
    destroy: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !ping_server(args.ip).await {
        anyhow::bail!("Could not reach server at {}. Is Port 22 open?", args.ip);
    }
    println!("Server is reachable");

    validate_key_file(&args.key_path)?;
    println!("SSH Key validated");

    let mut session = connect_ssh(args.ip, args.user, args.key_path).await?;
    println!("SSH connection established");

    if args.destroy {
        run_remote_cmd(&session, "sudo wg-quick down wg0 || true")?;
        run_remote_cmd(&session, "sudo rm -rf /etc/wireguard/")?;

        println!("VPS iS clean");

        return Ok(());
    }

    // let new_peer_config = add_new_peer(&session, args.ip, "test".into())?;

    // println!("added new peer with config");
    // println!("{}", new_peer_config);

    wireguard::server::setup_wireguard(&session, args.ip, &args.interface)?;
    harden_ssh(&session)?;

    println!("Setup complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use vpn_lib::KeyFileError;

    use super::*;
    use std::io::Write;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_ping_timeout() {
        let ip = Ipv4Addr::new(192, 0, 2, 1);
        let result = ping_server(ip).await;
        assert_eq!(result, false, "Should return false");
    }

    #[test]
    fn test_valid_file() {
        let mut temp_file = tempfile::NamedTempFile::new().unwrap();
        writeln!(temp_file, "some-data").unwrap();

        let path = temp_file.path().to_path_buf();
        let result = validate_key_file(&path);

        assert!(result.is_ok(), "Should accept a valid file");
    }

    #[test]
    fn test_missing_file_fails() {
        let path = PathBuf::from("non_existing_file.txt");
        let result = validate_key_file(&path);

        assert!(matches!(result, Err(KeyFileError::NotFound(_))));
    }
}
