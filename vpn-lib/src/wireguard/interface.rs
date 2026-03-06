use std::net::Ipv4Addr;

pub fn get_interface_index(virtual_ip: Ipv4Addr) -> Result<u32, String> {

	let interfaces = netdev::get_interfaces();
	let tun_index = interfaces.iter()
		.find(|i| i.ipv4.iter().any(|addr| addr.addr() == virtual_ip))
		.map(|i| i.index)
		.ok_or("WireGuard interface not found")?;

	Ok(tun_index)

}