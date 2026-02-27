use std::{net::Ipv4Addr, str::FromStr, sync::Mutex};
use tauri::{AppHandle, Emitter, State};
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use vpn_lib::network::ping_endpoint;

use crate::TunnelState;

pub struct PingHandle(pub Mutex<Option<CancellationToken>>);

#[tauri::command]
pub async fn start_ping_loop(
    app: AppHandle,
    state: State<'_, TunnelState>,
    stop_signal: State<'_, PingHandle>,
) -> Result<(), String> {
    let ip_str = {
        let lock = state
            .active_tunnel
            .lock()
            .map_err(|_| "Failed to lock tunnel state")?;
        lock.clone().ok_or("No active tunnel found to ping")?
    };

    let addr = Ipv4Addr::from_str(&ip_str).map_err(|_| "Invalid IP address in tunnel state")?;

    let token = CancellationToken::new();
    let child_token = token.child_token();

    let mut lock = stop_signal.0.lock().unwrap();
    *lock = Some(token);

    tauri::async_runtime::spawn(async move {
        loop {
            if child_token.is_cancelled() {
                break;
            }

            if let Some(latency) = ping_endpoint(addr).await {
                app.emit("ping-result", (addr.to_string(), latency))
                    .unwrap();
            }
            sleep(Duration::from_secs(2)).await;
        }
        app.emit("ping-stopped", addr.to_string()).unwrap();
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_ping_loop(stop_signal: State<'_, PingHandle>) -> Result<(), String> {
    let mut lock = stop_signal.0.lock().unwrap();
    if let Some(token) = lock.take() {
        token.cancel();
    }
    Ok(())
}
