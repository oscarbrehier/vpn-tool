use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    println!("cargo:rerun-if-changed=../../sidecar/src");
    println!("cargo:rerun-if-changed=../../sidecar/Cargo.toml");

    let cargo_build = std::process::Command::new("cargo")
        .args(&["build", "-p", "sidecar"])
        .args(if profile == "release" {
            vec!["--release"]
        } else {
            vec![]
        })
        .current_dir("../../")
        .status()
        .expect("Failed to build sidecar");

    if !cargo_build.success() {
        panic!("Sidecar build failed!");
    }

    let sidecar_exe = PathBuf::from("../../target")
        .join(&profile)
        .join(if cfg!(windows) {
            "sidecar.exe"
        } else {
            "sidecar"
        });

    if !sidecar_exe.exists() {
        panic!("Sidecar binary not found at {:?}", sidecar_exe);
    }

    let ext = if target.contains("windows") {
        ".exe"
    } else {
        ""
    };
    
    let sidecar_name = format!("sidecar-{}{}", target, ext);

    std::fs::create_dir_all("binaries").unwrap();

    let bundle_path = PathBuf::from("binaries").join(&sidecar_name);
    std::fs::copy(&sidecar_exe, &bundle_path).expect("Failed to copy sidecar to binaries/");

    let dev_path = PathBuf::from("../../target")
        .join(&profile)
        .join(&sidecar_name);

    std::fs::copy(&sidecar_exe, &dev_path).expect("Failed to copy sidecar to target/");

    tauri_build::build();
}
