use std::{collections::HashMap, net::Ipv4Addr};

use anyhow::Ok;
use etherparse::SlicedPacket;
use tokio::sync::mpsc;
use windivert::{layer::NetworkLayer, packet::WinDivertPacket};

type ConnectionMap = HashMap<u16, Ipv4Addr>;

fn handle_outbound(
    packet: &mut WinDivertPacket<'_>,
    virtual_ip: Ipv4Addr,
    interface_i: u32,
    connections: &mut ConnectionMap,
) -> Result<(), String> {
    let data = &mut packet.data;

    if data.len() < 20 {
        return Err("Packet too short".into());
    }

    let version = data[0] >> 4;
    if version != 4 {
        return Err("IPv6 is not supporteed".into());
    }

    let original_ip = Ipv4Addr::from([data[12], data[13], data[14], data[15]]);

    let ip_bytes = virtual_ip.octets();
    data[12..16].copy_from_slice(&ip_bytes);

    let transport_header_start = 20;

    if data.len() < transport_header_start + 2 {
        return Err("Packet too short for transport header".into());
    }

    let source_port = u16::from_be_bytes([
        data[transport_header_start],
        data[transport_header_start + 1],
    ]);

    connections.insert(source_port, original_ip);

    packet.address.if_idx = interface_i;
    packet.address.set_outbound(true);

    packet.recalculate_checksums()?;

    Ok(())
}

fn handle_inbound(
    packet: &mut WinDivertPacket<'_>,
    connections: &ConnectionMap,
) -> Result<(), String> {
    let data = &mut packet.data;

    if data.len() < 20 {
        return Err("Packet too short".into());
    }

    let version = data[0] >> 4;
    if version != 4 {
        return Err("IPv6 is not supported".to_string());
    }

    let transport_header_start = 20;

    if data.len() < transport_header_start + 2 {
        return Err("Packet too short for transport header".into());
    }

    let dest = u16::from_be_bytes([
        data[transport_header_start],
        data[transport_header_start + 1],
    ]);

    if let Some(original_ip) = connections.get(&dest) {
        
		let ip_bytes = virtual_ip.octets();
        data[12..16].copy_from_slice(&ip_bytes);

		packet.address.if_idx = 0;
		packet.address.set_outbound(false);

		packet.recalculate_checksums()?;

    } else {
        return Err("Application not found with specified port".into());
    }

    Ok(())
}

fn build_filter_string(pids: &[u32]) -> String {
    if pids.is_empty() {
        return "false".into();
    };

    let pid_conditions: Vec<String> = pids
        .iter()
        .map(|pid| format!("processId === {}", pid))
        .collect();

    format!("outbound and ({})", pid_conditions.join(" or "))
}

#[cfg(target_os = "windows")]
pub async fn start_packet_redirection(
    initial_pids: Vec<u32>,
    mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
	virtual_ip: Ipv4Addr,
	interface_i: u32
) -> Result<(), String> {
    use windivert::{WinDivert, prelude::WinDivertFlags};

	let mut connections = ConnectionMap::new();

    let current_filter = build_filter_string(&initial_pids);

    let mut divert = WinDivert::network(&current_filter, 0, WinDivertFlags::default())
        .map_err(|e| format!("Failed to open windivert: {}", e))?;

    let mut buffer = [0u8; 65535];

    loop {
        tokio::select! {

            Some(new_pids) = filter_rx.recv() => {

                current_filter = build_filter_string(&new_pids);

                // reopen with new filters
                divert = WinDivert::network(&current_filter, 0, WinDivertFlags::default()).map_err(|e| format!("Failed to open windivert: {}", e))?;

            }

            packet = async { divert.recv(Some(&mut buffer)) } => {

                if let Ok(wd_packet) = packet {

                    let mut modified_packet = wd_packet;

                    divert.send(&modified_packet).ok();

                }

            }

        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn start_packet_redirection(
    target_pids: Vec<u32>,
    mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
) -> Result<(), String> {
    // utun
}

#[cfg(target_os = "linux")]
pub async fn start_packet_redirection(
    target_pids: Vec<u32>,
    mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
) -> Result<(), String> {
    // netlink
}

pub fn update_filter_rules(
    tx: &mpsc::UnboundedSender<Vec<u32>>,
    pids: Vec<u32>,
) -> Result<(), String> {
    tx.send(pids).map_err(|e| e.to_string())
}
