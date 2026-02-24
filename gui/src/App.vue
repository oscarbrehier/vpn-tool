<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import "./assets/globals.css";
import Map from "./components/Map.vue";
import { Settings as SettingsIcon, X } from "lucide-vue-next";
import Settings from "./components/Settings.vue";
import { invoke } from "@tauri-apps/api/core";
import { getGeoLocation } from "./lib/geo";
import { listen } from "@tauri-apps/api/event";

interface TunnelPayload {
	name: string;
	is_active: boolean;
}

const isConnected = ref(false);
const isConnecting = ref(false);
const isSettingsOpen = ref(false);
const activeTunnel = ref<string | null>(null);

const locationData = ref({
	city: "",
	country: "",
	isp: ""
});

const availableEndpoints = ref<any[]>([]);

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

async function getAvailableEndpoints() {

	const confs: String[] = await invoke("get_configs");

	const locationsPromises = confs.map(async (conf) => {

		const lastIndex = conf.lastIndexOf('.');
		const ip = lastIndex !== -1 ? conf.substring(0, lastIndex) : conf;

		const res = await getGeoLocation(ip.toString());
		return res;

	});

	const locations = await Promise.all(locationsPromises);
	const validLocations = locations.filter(Boolean);

	return validLocations;

};

onMounted(async () => {

	try {

		const status = await invoke<TunnelPayload>('get_current_tunnel_status');

		isConnected.value = status.is_active;
		activeTunnel.value = status.name;

	} catch (err) { }

	await listen("tunnel-status", (event: { payload: TunnelPayload }) => {
		isConnected.value = event.payload.is_active;
		activeTunnel.value = event.payload.name;
	});

	const endpoints = await getAvailableEndpoints();
	availableEndpoints.value = endpoints;

});

watch(() => isConnected.value, async (connected) => {

	if (connected && activeTunnel.value) {

		const data = await getGeoLocation();

		if (data) {

			locationData.value = {
				...data
			};

		};

	};

}, { immediate: true });

const openSettings = () => isSettingsOpen.value = true;
const closeSettings = () => isSettingsOpen.value = false;

</script>

<template>

	<main class="h-screen w-screen bg-[#19272a] bg-cover bg-center">

		<!-- gradient -->
		<div class="absolute h-full w-full bg-linear-to-b via-transparent to-black/10 z-20 pointer-events-none transition-colors duration-1000"
			:class="isConnected ? 'from-emerald-500/30' : 'from-red-500/30'" />

		<!-- toolbar -->
		<div class="h-20 w-auto absolute top-0 right-0 z-60 p-4">

			<button v-if="!isSettingsOpen" @click="openSettings">
				<SettingsIcon />
			</button>

			<button v-else="isSettingsOpen" @click="closeSettings">
				<X />
			</button>

		</div>

		<div class="absolute z-50 bottom-0 left-0 w-full p-4 flex flex-col items-center">

			<button @click="handleToggle" class="h-12 rounded-md px-14 border border-neutral-500/20 mb-4"
				:class="isConnected ? 'bg-neutral-700' : 'bg-violet-700'">
				{{ isConnected ? "Disconnect" : "Connect" }}
			</button>

			<div class="h-auto w-full z-50  px-4 py-2 flex justify-between">
				<!-- <div class="h-auto bg-neutral-900 w-full z-50 rounded-md border border-neutral-500/20 px-4 py-2 flex justify-between"> -->

				<div>
					<p class="text-[12px] text-neutral-400">Your IP Address</p>
					<p class="text-sm">{{ activeTunnel || 'Detecting...' }}</p>
				</div>

				<div class="h-full w-px border border-neutral-500/20 mx-8" />

				<div>
					<p class="text-[12px] text-neutral-400">Country</p>
					<p class="text-sm">{{ locationData.country || 'Detecting...' }}</p>
				</div>

				<div class="h-full w-px border border-neutral-500/20 mx-8" />

				<div>
					<p class="text-[12px] text-neutral-400">Provider</p>
					<p class="text-sm">{{ locationData.isp || 'Detecting...' }}</p>
				</div>

			</div>

		</div>

		<!-- connection toggle -->
		<!-- <button
			@click="handleToggle"
			class="bg-violet-700 z-20 absolute left-1/2 -translate-x-1/2 -translate-y-1/2"
		>
			
		</button> -->

		<!-- connection info -->
		<!-- <div
			class="absolute h-20 w-full bottom-0 right-0 z-30 pointer-events-none flex items-center justify-between px-4">

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

		</div> -->

		<Map :tunnel="activeTunnel" :isConnected="isConnected" />

		<Settings :isOpen="isSettingsOpen" @close="isSettingsOpen = false" />

	</main>

</template>