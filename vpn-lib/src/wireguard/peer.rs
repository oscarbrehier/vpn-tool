use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use ssh2::Session;
use std::fmt::{self};
use std::net::Ipv4Addr;

use crate::wireguard::server::{build_client_config, update_wireguard_config};
use crate::wireguard::{
    server::generate_keys,
    state::{get_or_create_state, save_state},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Peer {
    pub name: String,
    pub public_key: String,
    pub ip: Ipv4Addr,
    pub crated_at: DateTime<Utc>,
}

impl Peer {
    pub fn new(name: String, ip: Ipv4Addr) -> (Self, String) {
        let (priv_key, pub_key) = generate_keys();
        (
            Self {
                name,
                public_key: pub_key,
                ip: ip,
                crated_at: Utc::now(),
            },
            priv_key,
        )
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "# Peer: {}\n[Peer]\nPublicKey = {}\nAllowedIPs = {}/32\n",
            self.name, self.public_key, self.ip
        )
    }
}

pub fn add_new_peer(
    session: &Session,
    server_ip: Ipv4Addr,
    name: String,
) -> anyhow::Result<String> {
    let mut state = get_or_create_state(session, server_ip)?;

    let next_ip = state.get_next_available_ip()?;
    let (new_peer, priv_key) = Peer::new(name, next_ip);

    state.peers.push(new_peer);
    state.last_updated = Utc::now();

    save_state(session, &state)?;
    update_wireguard_config(session, &state)?;

    let client_config = build_client_config(&priv_key, &state.server_public_key, state.server_ip, next_ip);

    anyhow::Ok(client_config)
}
