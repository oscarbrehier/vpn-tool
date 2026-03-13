use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    let profile = env::var("PROFILE").unwrap();

    let status = std::process::Command::new("cargo")
        .args(&["build", "--manifest-path", "../../vpn-sidecar/Cargo.toml"])
        .args(if profile == "release" {
            vec!["--release"]
        } else {
            vec![]
        })
        .status()
        .expect("Failed to build sidecar");

    if !status.success() {
        panic!("Failed to build vpn-sidecar");
    }

    let sidecar_src = PathBuf::from("../../vpn-sidecar/target")
        .join(&profile)
        .join(if cfg!(windows) {
            "vpn-sidecar.exe"
        } else {
            "vpn-sidecar"
        });

    let sidecar_dest = PathBuf::from("binaries").join(if cfg!(windows) {
        "vpn-sidecar.exe"
    } else {
        "vpn-sidecar"
    });

    std::fs::create_dir_all("binaries").ok();
    std::fs::copy(&sidecar_src, &sidecar_dest).expect("Failed to copy sidecar binary");

    let mut windows = tauri_build::WindowsAttributes::new();

    windows = windows.app_manifest(
        r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#,
    );

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run build script");
}
