import { GeoLocation, getGeoLocation } from "./geo";
import { runCommand } from "./tauri";

export type TunnelMode = "full" | "split"

export interface TunnelStatus {
	name: string | null;
	is_active: boolean;
};

export interface TunnelMetadata {
	client_ip: string;
	name: string;
	server_public_key: string;
	public_ip: string;
	location: GeoLocation;
};

export interface UnifiedEndpoint {
	geo: GeoLocation;
	config: TunnelMetadata;
};

export async function getConfigurations(): Promise<TunnelMetadata[]> {

	const { data: confs, error } = await runCommand<Omit<TunnelMetadata, "location">[]>("get_configs", true);

	if (error || !confs) return [];

	const locationsPromises = confs.map(async (conf) => {

		const ip = conf.name.split("-")[1];

		const res = await getGeoLocation(ip);

		if (!res) return null;

		return {
			...conf,
			location: res
		};

	});

	const results = await Promise.all(locationsPromises);
	return results.filter((item) => item !== null);

};

export async function startTunnel(conf: TunnelMetadata, mode: TunnelMode) {
	await runCommand("start_tunnel", true, {
		publicIp: conf.public_ip,
		tunnelMode: mode
	});
};

export async function stopTunnel(): Promise<{ error: string | null }> {
	return await runCommand("stop_tunnel", true);
};

export async function quickConnect(mode: TunnelMode): Promise<boolean> {

	const { data, error } = await runCommand<{ config_name: string, success: boolean }>("quick_connect", true, {
		tunnelMode: mode
	});

	if (error) {
		return false;
	};

	return data?.success ?? false;

};

export async function getTunnelStatus() {
	return await runCommand<TunnelStatus>("get_current_tunnel_status", true);
}

export async function getAvailableEndpoints(): Promise<UnifiedEndpoint[]> {

	const { data: confs, error } = await runCommand<TunnelMetadata[]>("get_configs", true);

	if (!error || !confs) return [];

	const locationsPromises = (confs as TunnelMetadata[]).map(async (conf) => {
 
		const ip = conf.name.split("-")[1];
		const res = await getGeoLocation(ip);

		if (!res) return null;

		return {
			geo: res,
			config: conf
		};

	});

	const results = await Promise.all(locationsPromises);
	return results.filter((i): i is UnifiedEndpoint => i !== null);

};