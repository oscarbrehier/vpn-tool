use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::{
    io::Read,
    net::{IpAddr, Ipv4Addr},
};

use crate::wireguard::peer::Peer;

#[derive(Deserialize, Debug, Serialize, Default)]
pub struct VpnState {
	pub server_public_key: String,
	pub server_ip: String,
    pub peers: Vec<Peer>,
	pub last_updated: DateTime<Utc>
}

#[derive(Debug, thiserror::Error)]
pub enum StateError {
	#[error("The network is full (maximum 253 peers reached)")]
    NetworkFull,
}

impl VpnState {
    pub fn get_next_available_ip(&self) -> Result<Ipv4Addr, StateError> {
        let base_ip = [10, 0, 0, 0];

        let max_octet = self
            .peers
            .iter()
            .map(|p| p.ip.octets()[3])
            .max()
            .unwrap_or(1);

        if max_octet >= 254 {
            return Err(StateError::NetworkFull);
        }

        Ok(Ipv4Addr::new(
            base_ip[0],
            base_ip[1],
            base_ip[2],
            max_octet + 1,
        ))
    }
}

pub fn get_or_create_state(session: &Session) -> anyhow::Result<VpnState> {
    let cmd = "cat /etc/wireguard/peers.json";

    let mut channel = session.channel_session()?;
    channel.exec(cmd)?;

    let mut contents = String::new();
    channel.read_to_string(&mut contents)?;

    if contents.is_empty() {
        anyhow::Ok(VpnState::default())
    } else {
        anyhow::Ok(serde_json::from_str(&contents)?)
    }
}

pub fn save_state(session: &Session, state: &VpnState) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(state)?;
    let escaped_json = json.replace("'", "'\\''");

    let cmd = format!(
        "echo '{}' | sudo tee /etc/wireguard/peers.json",
        escaped_json
    );

    let mut channel = session.channel_session()?;
    channel.exec(&cmd)?;
    channel.wait_close()?;

    anyhow::Ok(())
}
