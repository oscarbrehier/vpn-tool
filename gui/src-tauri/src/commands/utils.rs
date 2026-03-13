use secrecy::{ExposeSecret, SecretString};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::{net::Ipv4Addr, process::Command};
use tauri::AppHandle;

pub async fn save_key_securely(
    app: &AppHandle,
    public_ip: Ipv4Addr,
    private_key: &SecretString,
) -> Result<(), String> {
    let account_name = format!("priv_key_{}", public_ip);
    println!(
        "DEBUG: Attempting to SAVE key for account: {}",
        account_name
    );

    #[cfg(target_os = "windows")]
    {
        use std::fs;

        use tauri::Manager;
        use windows_dpapi::{encrypt_data, Scope};

        let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
        let key_path = app_dir.join(format!("{}.enc", account_name));

        let encrypted = encrypt_data(private_key.expose_secret().as_bytes(), Scope::User)
            .map_err(|e| format!("Windows Encryption Failed: {}", e))?;

        fs::write(key_path, encrypted).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        let entry =
            keyring::Entry::new("com.vpnapp.keys", &account_name).map_err(|e| e.to_string())?;

        entry
            .set_password(private_key.expose_secret())
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn load_key_securely(app: &AppHandle, public_ip: Ipv4Addr) -> Result<SecretString, String> {
    let account_name = format!("priv_key_{}", public_ip);
    println!(
        "DEBUG: Attempting to LOAD key for account: {}",
        account_name
    );

    #[cfg(target_os = "windows")]
    {
        use std::fs;

        use tauri::Manager;
        use windows_dpapi::{Scope, decrypt_data};

        let app_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
        let key_path = app_dir.join(format!("{}.enc", account_name));

        let encrypted = fs::read(key_path).map_err(|_| "Key file not found".to_string())?;
        let decrypted = decrypt_data(&encrypted, Scope::User).map_err(|e| format!("Windows Decryption Failed: {}", e))?;

        Ok(SecretString::new(String::from_utf8(decrypted).unwrap().into()))
    }

    #[cfg(not(target_os = "windows"))]
    {
        let entry =
            keyring::Entry::new("com.vpnapp.keys", &account_name).map_err(|e| e.to_string())?;

        let password = entry.get_password().map_err(|e| e.to_string())?;

        Ok(SecretString::new(password.into()))
    }
}
