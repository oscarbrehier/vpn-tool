use clap::{Parser};
use ssh2::{Session};
use std::{
    io::{Write},
    net::{IpAddr},
    path::{PathBuf},
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};
use vpn_tool::wireguard::{self};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    ip: IpAddr,
    #[arg(short, long)]
    key_path: PathBuf,
    #[arg(short, long, default_value = "dev")]
    user: String,
    #[arg(long, default_value = "eth0")]
    interface: String
}

#[derive(Debug, thiserror::Error)]
pub enum KeyFileError {
    #[error("Key file not found at {0}")]
    NotFound(PathBuf),
    #[error("The path provided is a directory, not a file")]
    IsDirectory,
    #[error("Key file doesn't have the correct permissions to read: {0}")]
    NoReadPermissions(String),
    #[error("Could not read metadata: {0}")]
    ParseMetadata(String),
    #[error("File system error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum SshError {
    #[error("Failed to reach server at {0}: {1}")]
    Network(IpAddr, std::io::Error),
    #[error("SSH handshake failed: {0}")]
    Protocol(#[from] ssh2::Error),
    #[error("Authentication failed for user {0}. Check your private key")]
    AuthFailed(String),
    #[error("The background task panicked")]
    ThreadError(#[from] tokio::task::JoinError),
}

async fn ping_server(addr: &IpAddr) -> bool {
    match timeout(Duration::from_secs(3), TcpStream::connect((*addr, 22))).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

fn validate_key_file(path: &PathBuf) -> Result<(), KeyFileError> {
    if !path.exists() {
        return Err(KeyFileError::NotFound(path.clone()));
    }

    let metadata = path.metadata()?;

    if metadata.is_dir() {
        return Err(KeyFileError::IsDirectory);
    };

    std::fs::read(path).map_err(|e| KeyFileError::NoReadPermissions(e.to_string()))?;

    Ok(())
}

async fn connect_ssh(addr: IpAddr, user: String, key_path: PathBuf) -> Result<Session, SshError> {
    tokio::task::spawn_blocking(move || {
        let stream =
            std::net::TcpStream::connect((addr, 22)).map_err(|e| SshError::Network(addr, e))?;

        let mut sess = Session::new()?;
        sess.set_tcp_stream(stream);
        sess.handshake()?;

        sess.userauth_pubkey_file(&user, None, &key_path, None)?;

        if !sess.authenticated() {
            return Err(SshError::AuthFailed(user));
        }

        Ok(sess)
    })
    .await?
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if !ping_server(&args.ip).await {
        anyhow::bail!("Could not reach server at {}. Is Port 22 open?", args.ip);
    }
    println!("Server is reachable");

    validate_key_file(&args.key_path)?;
    println!("SSH Key validated");

    let mut session = connect_ssh(args.ip, args.user, args.key_path).await?;
    println!("SSH connection established");

    wireguard::setup_wireguard(&session, &args.ip, &args.interface)?;
    
    println!("Setup complete");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::net::Ipv4Addr;

    #[tokio::test]
    async fn test_ping_timeout() {
        let ip = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1));
        let result = ping_server(&ip).await;
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
