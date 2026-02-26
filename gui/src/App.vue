<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import "./assets/globals.css";
import Map from "./components/Map.vue";
import Settings from "./components/Settings.vue";
import { invoke } from "@tauri-apps/api/core";
import { getGeoLocation } from "./lib/geo";
import { listen } from "@tauri-apps/api/event";
import { quickConnect, stopTunnel } from "./lib/tunnel";
import { Toaster } from 'vue-sonner';
import 'vue-sonner/style.css'
import Toolbar from "./components/Toolbar.vue";
import { startPinging, stopPinging } from "./lib/network";


interface TunnelPayload {
	name: string;
	is_active: boolean;
};

const isConnected = ref(false);
const isSettingsOpen = ref(false);
const activeTunnel = ref<string | null>(null);
const mapFocusIp = ref<string | null>(null);

const locationData = ref({
	country: "",
	as_name: ""
});

const networkData = ref<{ latency: null | number }>({
	latency: null
});

const availableEndpoints = ref<any[]>([]);

async function handleToggle() {

	if (isConnected.value) {
		await stopTunnel();
	} else {
		await quickConnect();
	};

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

	await listen("ping-result", (event: { payload: [string, number] }) => {
		networkData.value.latency = event.payload[1];
	});

	await listen("ping-stopped", () => {
		networkData.value.latency = null;
	});

	const endpoints = await getAvailableEndpoints();
	availableEndpoints.value = endpoints;

});

watch(() => isConnected.value, async (connected) => {

	const data = await getGeoLocation();

	if (data) {

		locationData.value = { ...data };

		if (!connected) {
			mapFocusIp.value = data.ip;
		} else {
			mapFocusIp.value = activeTunnel.value;
		}

	};

	if (connected) {
		startPinging();
	} else {
		stopPinging();
	};

}, { immediate: true });

const openSettings = () => isSettingsOpen.value = true;
const closeSettings = () => isSettingsOpen.value = false;

</script>

<template>

	<Toaster class="background-blur-xl" :toastOptions="{
		class: 'backdrop-blur-xl !bg-[#19272a]/60 border-t border-white/20 border-x border-b border-white/5'
	}" :closeButton="true" closeButtonPosition="top-right" position="top-left" theme="dark" richColors />

	<main class="h-screen w-scree bg-cover bg-center">

		<Map :tunnel="mapFocusIp" :isConnected="isConnected" />


		<!-- gradient -->
		<div class="absolute h-full w-full bg-linear-to-b via-transparent to-black/10 z-20 pointer-events-none transition-colors duration-1000"
			:class="isConnected ? 'from-accent-500/30' : 'from-brand-500/30'" />

		<Toolbar :isOpen="isSettingsOpen" v-on:open="openSettings" v-on:close="closeSettings" />

		<div class="absolute z-50 bottom-0 left-0 w-full p-4 flex flex-col items-center">

			<!-- <button class="capitalize px-8 py-2 rounded-full bg-slate-700">
				{{ isConnected ? "connect" : "disconnect" }}
			</button> -->

			<div class="inline-block p-[2px] rounded-full bg-linear-to-r from-brand-400 via-zinc-700 to-zinc-600">
				<button class="px-6 py-4 bg-background rounded-full uppercase font-semibold text-zinc-300">
					{{ isConnected ? "disconnect" : "connect" }}
				</button>
			</div>

		</div>


		<Settings :isOpen="isSettingsOpen" @close="isSettingsOpen = false" />

	</main>

</template>