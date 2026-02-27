use std::net::Ipv4Addr;
use secrecy::{ExposeSecret, SecretString};

pub async fn save_key_securely(
    public_ip: Ipv4Addr,
    private_key: &SecretString,
) -> Result<(), String> {
    let entry = keyring::Entry::new("vpn_app", &format!("priv_key_{}", public_ip))
        .map_err(|e| e.to_string())?;

    entry
        .set_password(private_key.expose_secret())
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub async fn load_key_securely(public_ip: Ipv4Addr) -> Result<SecretString, String> {
    let entry = keyring::Entry::new("vpn_app", &format!("priv_key_{}", public_ip))
        .map_err(|e| e.to_string())?;

	let password = entry.get_password().map_err(|e| e.to_string())?;

    Ok(SecretString::new(password.into()))
}
