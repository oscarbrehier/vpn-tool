import { GeoLocation, getGeoLocation } from "./geo";
import { runCommand } from "./tauri";

export interface TunnelMetadata {
	client_ip: string;
	name: string;
	server_public_key: string;
	public_ip: string;
	location: GeoLocation;
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

export async function stopTunnel(): Promise<{ error: string | null }> {
	return await runCommand("stop_tunnel", true);
};

export async function quickConnect(): Promise<boolean> {

	const { data, error } = await runCommand<{ config_name: string, success: boolean }>("quick_connect", true);

	if (error) {
		return false;
	};

	return data?.success ?? false;

};