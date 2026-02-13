pub mod ssh;
pub mod wireguard;

use ssh2::Session;
use std::{net::IpAddr, path::PathBuf, time::Duration};
use tokio::{net::TcpStream, time::timeout};

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

pub async fn ping_server(addr: &IpAddr) -> bool {
    match timeout(Duration::from_secs(3), TcpStream::connect((*addr, 22))).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

pub fn validate_key_file(path: &PathBuf) -> Result<(), KeyFileError> {
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

pub async fn connect_ssh(addr: IpAddr, user: String, key_path: PathBuf) -> Result<Session, SshError> {
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
