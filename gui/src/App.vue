<script setup lang="ts">
import { onMounted, ref } from "vue";
import "./assets/globals.css";
import Map from "./components/Map.vue";

const isConnected = ref(false);
const isConnecting = ref(false);

async function handleToggle() {
	if (isConnected.value) {
		isConnected.value = false
		return
	}

	isConnecting.value = true
	await new Promise(resolve => setTimeout(resolve, 2000))
	isConnecting.value = false
	isConnected.value = true
};

const locationData = ref({
	ip: "",
	city: "",
	country: "",
	lat: 0,
	lon: 0,
	isp: ""
});

async function getGeoLocation() {

	try {

		const res = await fetch(`http://ip-api.com/json/`);
		const data = await res.json();

		if (data.status == "success") {
			locationData.value = {
				ip: data.query,
				...data
			};
		}

	} catch (err) {

	}

}

onMounted(async () => {
	await getGeoLocation();
});

</script>

<template>

	<main class="h-screen w-screen bg-[#19272a] bg-cover bg-center">

		<div
			class="absolute h-full w-full bg-linear-to-b from-red-500/30 via-transparent to-black/10 z-20 pointer-events-none">
		</div>

		<div class="absolute h-20 w-full bottom-0 z-30 pointer-events-none flex items-center justify-between px-4">

			<div>
				<p class="text-sm text-neutral-400">Your IP Address</p>
				<p>{{ locationData.ip || 'Detecting...' }}</p>
			</div>

			<div>
				<p class="text-sm text-neutral-400">Country</p>
				<p>{{ locationData.country || 'Detecting...' }}</p>
			</div>

			<div>
				<p class="text-sm text-neutral-400">Provider</p>
				<p>{{ locationData.isp || 'Detecting...' }}</p>
			</div>

		</div>

		<Map :lat="locationData.lat" :lon="locationData.lon" :country="locationData.country" />

		<!-- <div
			class="h-full w-[35vw] bg-neutral-800/30 backdrop-blur-xl p-4 rounded-md border border-neutral-100/10 flex flex-col justify-between">

			<div></div>

			<div>

				<div class="flex flex-col gap-2">

					<input placeholder="Host/IP" class="bg-neutral-800 w-full rounded px-4 py-2 outline-none" />

					<input placeholder="Username" class="bg-neutral-800 w-full rounded px-4 py-2 outline-none" />

					<input placeholder="SSH Key" class="bg-neutral-800 w-full rounded px-4 py-2 outline-none" />

				</div>

				<button class="h-10 w-full rounded bg-[#0652DD] mt-4">
					Configure
				</button>

			</div>

		</div> -->

	</main>

</template>