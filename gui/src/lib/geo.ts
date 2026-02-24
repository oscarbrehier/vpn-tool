export interface GeoLocation {
	status: string;
	country: string;
	countryCode: string;
	regionName: string;
	city: string;
	lat: number;
	lon: number;
	timezone: string;
	isp: string;
	query: string;
};

export async function getGeoLocation(ip_address?: string): Promise<GeoLocation | null> {

	try {

		let url = `http://ip-api.com/json`;
		if (ip_address) url += `/${ip_address}`;

		const res = await fetch(url);
		const data = await res.json();

		return data;

	} catch (err) {
		return null;
	}

};
