// use std::{collections::HashMap, net::Ipv4Addr};
// use tokio::sync::mpsc;
// use windivert::{layer::NetworkLayer, packet::WinDivertPacket};
// use windivert_sys::ChecksumFlags;

// type ConnectionMap = HashMap<u16, Ipv4Addr>;

// fn is_non_routable(ip: Ipv4Addr) -> bool {
//     let octets = ip.octets();
//     ip.is_loopback()
//         || ip.is_multicast()
//         || octets[0] == 224
//         || octets[0] == 239
//         || ip.is_link_local()
//         || ip.is_broadcast()
// }

// fn handle_outbound(
//     packet: &mut WinDivertPacket<'_, NetworkLayer>,
//     wg_server_internal_ip: Ipv4Addr,
//     connections: &mut ConnectionMap,
// ) -> anyhow::Result<()> {
//     let data = packet.data.to_mut();

//     if data.len() < 20 {
//         anyhow::bail!("Packet too short");
//     }

//     if data[0] >> 4 != 4 {
//         anyhow::bail!("Not IPv4");
//     }

//     let ihl = (data[0] & 0x0F) as usize * 4;

//     if data.len() < ihl + 4 {
//         anyhow::bail!("No transport header");
//     }

//     let real_dest = Ipv4Addr::from([data[16], data[17], data[18], data[19]]);

//     if is_non_routable(real_dest) {
//         anyhow::bail!("Skip non-routable dst {}", real_dest);
//     }

//     let src_port = u16::from_be_bytes([data[ihl], data[ihl + 1]]);

//     connections.insert(src_port, real_dest);

//     data[16..20].copy_from_slice(&wg_server_internal_ip.octets());

//     packet.address.set_impostor(true);

//     packet.recalculate_checksums(ChecksumFlags::new())?;

//     println!(
//         "out: src_port={} real_dest={} -> via wireguard",
//         src_port, real_dest
//     );

//     anyhow::Ok(())
// }

// fn handle_inbound(
//     packet: &mut WinDivertPacket<'_, NetworkLayer>,
//     connections: &ConnectionMap,
// ) -> anyhow::Result<()> {
//     let data = packet.data.to_mut();

//     if data.len() < 20 {
//         anyhow::bail!("Packet too short");
//     }

//     let ihl = (data[0] & 0x0F) as usize * 4;

//     if data.len() < ihl + 4 {
//         anyhow::bail!("No transport header");
//     }

//     let dest_port = u16::from_be_bytes([data[ihl + 2], data[ihl + 3]]);

//     if let Some(real_dest) = connections.get(&dest_port) {
//         data[12..16].copy_from_slice(&real_dest.octets());

//         packet.address.set_impostor(true);

//         packet.recalculate_checksums(ChecksumFlags::new())?;

//         println!("in : port={} restored src={}", dest_port, real_dest);
//     } else {
//         anyhow::bail!("No entry for dest_port {}", dest_port);
//     }

//     Ok(())
// }

// fn build_filter_string(wg_internal_ip: Ipv4Addr) -> String {
//     format!(
//         "(outbound and ip and not impostor) or (inbound and ip.SrcAddr == {} and not impostor)",
//         wg_internal_ip
//     )
// }

// #[cfg(target_os = "windows")]
// pub async fn start_packet_redirection(
//     initial_pids: Vec<u32>,
//     mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
//     wg_server_internal_ip: Ipv4Addr,
// ) -> anyhow::Result<()> {
//     use windivert::{WinDivert, layer::NetworkLayer, prelude::WinDivertFlags};

//     std::thread::spawn(move || {
//         let mut connections = ConnectionMap::new();
//         let mut buffer = [0u8; 65535];
//         let mut current_divert: Option<WinDivert<NetworkLayer>> = None;

//         let filter = build_filter_string(wg_server_internal_ip);

//         if !initial_pids.is_empty() {
//             current_divert = WinDivert::network(&filter, 0, WinDivertFlags::default()).ok();
//         }

//         loop {
//             if current_divert.is_none() {
//                 if let Some(new_pids) = filter_rx.blocking_recv() {
//                     if !new_pids.is_empty() {
//                         current_divert =
//                             WinDivert::network(&filter, 0, WinDivertFlags::default()).ok();
//                     }
//                 } else {
//                     break;
//                 }
//             }

//             if current_divert.is_some() {
//                 while let Ok(new_pids) = filter_rx.try_recv() {
//                     if new_pids.is_empty() {
//                         current_divert = None;
//                         break;
//                     }
//                 }
//             }

//             if let Some(divert) = current_divert.as_mut() {
//                 let mut wd_packet = match divert.recv(Some(&mut buffer)) {
//                     Ok(p) => p,
//                     Err(_) => continue,
//                 };

//                 let result = if wd_packet.address.outbound() {
//                     handle_outbound(&mut wd_packet, wg_server_internal_ip, &mut connections)
//                 } else {
//                     handle_inbound(&mut wd_packet, &connections)
//                 };

//                 if result.is_ok() {
//                     let _ = divert.send(&wd_packet);
//                 }
//             }
//         }
//     });

//     Ok(())
// }

// #[cfg(target_os = "macos")]
// pub async fn start_packet_redirection(
//     target_pids: Vec<u32>,
//     mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
// ) -> Result<(), String> {
//     Ok(())
// }

// #[cfg(target_os = "linux")]
// pub async fn start_packet_redirection(
//     target_pids: Vec<u32>,
//     mut filter_rx: mpsc::UnboundedReceiver<Vec<u32>>,
// ) -> Result<(), String> {
//     Ok(())
// }
