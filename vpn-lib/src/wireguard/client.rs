use std::{fs, path::Path, process::Command};

use anyhow::Context;

use crate::utils::create_command;

pub fn list_local_configs(conf_dir: &Path) -> anyhow::Result<Vec<String>> {
    let mut configs = Vec::new();

    if conf_dir.exists() && conf_dir.is_dir() {
        for entry in fs::read_dir(conf_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("conf") {
                if let Some(file_stem) = path.file_name().and_then(|s| s.to_str()) {
                    configs.push(file_stem.to_string());
                }
            }
        }
    }

    anyhow::Ok(configs)
}

pub fn start_tunnel(conf_path: &Path) -> anyhow::Result<()> {
    let path_str = conf_path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path"))?;

    let clean_path = path_str.strip_prefix(r#"\\?\"#).unwrap_or(path_str);

    println!("Cleaned Path for WireGuard: {}", clean_path);

    println!("{}", clean_path);

    let (bin, args) = if cfg!(target_os = "windows") {
        (
            "C:\\Program Files\\WireGuard\\wireguard.exe",
            vec!["/installtunnelservice", clean_path],
        )
    } else {
        ("wg-quick", vec!["up", conf_path.to_str().unwrap()])
    };

    let output = create_command(bin)
        .args(&args)
        .output()
        .context("Failed to execute WireGuard command")?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("WireGuard error: {}", err)
    }
}

pub fn stop_tunnel(conf_name: &str) -> anyhow::Result<()> {

    let (bin, args) = if cfg!(target_os = "windows") {
        (
            "C:\\Program Files\\WireGuard\\wireguard.exe",
            vec!["/uninstalltunnelservice", conf_name],
        )
    } else {
        ("wg-quick", vec!["down", conf_name])
    };

    let output = create_command(bin)
        .args(&args)
        .output()
        .context("Failed to execute WireGuard command")?;

    if output.status.success() {
        Ok(())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("WireGuard error: {}", err)
    }
}
