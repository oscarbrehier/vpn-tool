use std::{collections::HashMap, net::Ipv4Addr};
use tokio::sync::mpsc;
use windivert::{layer::NetworkLayer, packet::WinDivertPacket};
use windivert_sys::ChecksumFlags;

type ConnectionMap = HashMap<u16, Ipv4Addr>;

fn handle_outbound(
    packet: &mut WinDivertPacket<'_, NetworkLayer>,
    virtual_ip: Ipv4Addr,
    interface_i: u32,
    connections: &mut ConnectionMap,
) -> anyhow::Result<()> {
    let data = packet.data.to_mut();

    if data.len() < 20 {
        anyhow::bail!("Packet too short");
    }

    let version = data[0] >> 4;
    if version != 4 {
        anyhow::bail!("IPv6 is not supporteed");
    }

    let original_ip = Ipv4Addr::from([data[12], data[13], data[14], data[15]]);

    let ihl = (data[0] & 0x0F) as usize * 4;

    if data.len() < ihl + 4 {
        anyhow::bail!("No transport header");
    }

    let source_port = u16::from_be_bytes([data[ihl], data[ihl + 1]]);

    connections.insert(source_port, original_ip);

    let ip_bytes = virtual_ip.octets();
    data[12..16].copy_from_slice(&ip_bytes);

    packet.address.set_interface_index(interface_i);
    packet.address.set_outbound(true);

    packet.recalculate_checksums(ChecksumFlags::new())?;

    anyhow::Ok(())
}

fn handle_inbound(
    packet: &mut WinDivertPacket<'_, NetworkLayer>,
    connections: &ConnectionMap,
) -> anyhow::Result<()> {
    let data = packet.data.to_mut();

    if data.len() < 20 {
        anyhow::bail!("Packet too short");
    }

    let ihl = (data[0] & 0x0F) as usize * 4;

    if data.len() < ihl + 4 {
        anyhow::bail!("No transport header");
    }

    let dest_port = u16::from_be_bytes([data[ihl + 2], data[ihl + 3]]);

    if let Some(original_ip) = connections.get(&dest_port) {
        let ip_bytes = original_ip.octets();
        data[16..20].copy_from_slice(&ip_bytes);

        packet.address.set_interface_index(0);
        packet.address.set_outbound(false);

        packet.recalculate_checksums(ChecksumFlags::new())?;
    } else {
        anyhow::bail!("No tracking entry for port {}", dest_port);
    }

    Ok(())
}

fn build_filter_string(pids: &[u32], virtual_ip: Ipv4Addr) -> String {
    if pids.is_empty() {
        return "false".into();
    };

    let pid_conditions: String = pids
        .iter()
        .map(|pid| format!("processId === {}", pid))
        .collect::<Vec<_>>()
        .join(" or ");

    format!(
        "(outbound and ({})) or (inbound and ip.DstAddr == {})",
        pid_conditions, virtual_ip
    )
}

#[cfg(target_os = "windows")]
pub async fn start_packet_redirection(
    initial_pids: Vec<u32>,
    mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
    virtual_ip: Ipv4Addr,
    interface_i: u32,
) -> anyhow::Result<()> {
    use windivert::{WinDivert, prelude::WinDivertFlags};

    let mut connections = ConnectionMap::new();
    let mut current_filter = build_filter_string(&initial_pids, virtual_ip);

    let mut divert = WinDivert::network(&current_filter, 0, WinDivertFlags::default())
        .map_err(|e| anyhow::anyhow!("Windivert Error: {}", e))?;

    let mut buffer = [0u8; 65535];

    loop {
        tokio::select! {

            Some(new_pids) = filter_rx.recv() => {

                current_filter = build_filter_string(&new_pids, virtual_ip);

                // update divert handle with new filters
                divert = WinDivert::network(&current_filter, 0, WinDivertFlags::default()).map_err(|e| anyhow::anyhow!("Windivert Error: {}", e))?;

            }

            packet = async { divert.recv(Some(&mut buffer)) } => {

                let mut wd_packet = match packet {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                let result = if wd_packet.address.outbound() {
                    handle_outbound(&mut wd_packet, virtual_ip, interface_i, &mut connections)
                } else {
                    handle_inbound(&mut wd_packet, &connections)
                };

                if result.is_ok() {
                    let _ = divert.send(&wd_packet);
                }

            }

        }
    }
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
