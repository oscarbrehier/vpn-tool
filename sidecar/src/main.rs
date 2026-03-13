use std::io::{self, Write};

use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(name = "helper")]
#[command(about = "")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {}

#[derive(Serialize, Deserialize)]
struct Response {
    success: bool,
    message: String,
    data: Option<serde_json::Value>,
}

fn send_response(response: Response) {
    let json = serde_json::to_string(&response).unwrap();
    println!("{}", json);
    io::stdout().flush().unwrap();
}

#[cfg(windows)]
fn is_elevated() -> bool {
    use std::mem;
    use windows::Win32::Security::{
        GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token = Default::default();

        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size = 0u32;

        if GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_ok()
        {
            elevation.TokenIsElevated != 0
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

fn main() {
    let cli = Cli::parse();

    #[cfg(windows)]
    if !is_elevated() {
        send_response(Response {
            success: false,
            message: "Sidecar must run with aministrator privileges".to_string(),
            data: None,
        });
        std::process::exit(1);
    }

    let response = match cli.command {};

    send_response(response);
}
