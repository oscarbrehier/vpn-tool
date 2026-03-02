<script setup lang="ts">
import { ref, reactive, onMounted, watch, nextTick } from 'vue';
import { getConfigurations, startTunnel, TunnelMetadata } from '../lib/tunnel';
import { invoke } from '@tauri-apps/api/core';
import { getGeoLocation } from '../lib/geo';
import { runCommand } from '../lib/tauri';

const props = defineProps(['tunnel', 'isConnected']);

const svgContent = ref('');
const dotPos = reactive({ x: 0, y: 0 });
const isDragging = ref(false);

const allMarkers = ref<(TunnelMetadata & { x: 0, y: 0 })[]>([]);

const SVG_W = 2000;
const SVG_H = 857;

const transform = reactive({
	x: -2150,
	y: -621,
	scale: 3
});

const lastMousePos = { x: 0, y: 0 };

onMounted(async () => {
	try {
		const response = await fetch('/world.svg');
		const text = await response.text();
		svgContent.value = text.replace(/<svg[^>]*>|<\/svg>/g, '');
	} catch (err) {
		console.error("Failed to load map:", err);
	};
});

function startPan(e: MouseEvent) {
	isDragging.value = true;
	lastMousePos.x = e.clientX;
	lastMousePos.y = e.clientY;
};

function stopPan() {
	isDragging.value = false;
};

function doPan(e: MouseEvent) {

	if (!isDragging.value) return;

	const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();

	const ratio = Math.max(SVG_W / rect.width, SVG_H / rect.height);
	const dx = (e.clientX - lastMousePos.x) * ratio;
	const dy = (e.clientY - lastMousePos.y) * ratio;

	let newX = transform.x + dx;
	let newY = transform.y + dy;

	const mapW = SVG_W * transform.scale;
	const mapH = SVG_H * transform.scale;

	const minX = SVG_W - mapW;
	const minY = SVG_H - mapH;

	transform.x = Math.min(0, Math.max(newX, minX));
	transform.y = Math.min(0, Math.max(newY, minY));

	lastMousePos.x = e.clientX;
	lastMousePos.y = e.clientY;

};

function handleWheel(e: WheelEvent) {

	const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
	const ratio = Math.max(SVG_W / rect.width, SVG_H / rect.height);

	const SENSITIVITY = 0.002;
	const MIN_ZOOM = 3;
	const MAX_ZOOM = 8;

	const oldScale = transform.scale;

	let newScale = oldScale * (1 - e.deltaY * SENSITIVITY);
	newScale = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, newScale));

	const mouseX = (e.clientX - rect.left) * ratio;
	const mouseY = (e.clientY - rect.top) * ratio;

	const scaleRatio = newScale / oldScale;
	let nextX = mouseX - (mouseX - transform.x) * scaleRatio;
	let nextY = mouseY - (mouseY - transform.y) * scaleRatio;

	const mapW = SVG_W * newScale;
	const mapH = SVG_H * newScale;
	const minX = SVG_W - mapW;
	const minY = SVG_H - mapH;

	transform.x = Math.min(0, Math.max(nextX, minX));
	transform.y = Math.min(0, Math.max(nextY, minY));
	transform.scale = newScale;

};

function resetView() {
	transform.x = 0;
	transform.y = 0;
	transform.scale = 1;
};

function flyToCountry(countryName: string, zoomLevel = 6.5) {

	const coords = getCountryCenter(countryName);

	if (coords) {

		dotPos.x = coords.x;
		dotPos.y = coords.y;

		transform.scale = zoomLevel;

		const centerX = SVG_W / 2;
		const centerY = SVG_H / 2;

		transform.x = centerX - (coords.x * zoomLevel);
		transform.y = centerY - (coords.y * zoomLevel);

		const minX = SVG_W - (SVG_W * zoomLevel);
		const minY = SVG_H - (SVG_H * zoomLevel);

		transform.x = Math.min(0, Math.max(transform.x, minX));
		transform.y = Math.min(0, Math.max(transform.y, minY));

	};

};

async function placePointsOnMap() {

	const configs = await getConfigurations();

	const mapMarkers = configs.map((conf) => {

		if (!conf.location) return;

		const coords = getCountryCenter(conf.location.country);

		if (coords) {

			return {
				...conf,
				x: coords.x,
				y: coords.y
			}

		};

		return null;

	});

	allMarkers.value = mapMarkers.filter((m): m is any => m !== null);

}

function getCountryCenter(countryName: string): { x: number, y: number } | null {

	if (!countryName || !svgContent.value) return null;

	const nameWithUnderscore = countryName.replace(/\s+/g, '_');

	const selectors = [
		`.${nameWithUnderscore}`,
		`[id="${countryName}"]`,
		`[id="${nameWithUnderscore}"]`,
		`[name="${countryName}"]`,
		`[class*="${countryName}"]`
	];

	const elements = document.querySelectorAll(selectors.join(', '));

	let largestArea = 0;
	let bestCenter: { x: number, y: number } | null = null;

	elements.forEach((el) => {

		if (el instanceof SVGGraphicsElement) {

			const bbox = el.getBBox();
			const area = bbox.width * bbox.height;

			if (area > largestArea) {

				largestArea = area;
				bestCenter = {
					x: bbox.x + bbox.width / 2,
					y: bbox.y + bbox.height / 2
				};

			};

		};

	});

	return bestCenter;

};

onMounted(async () => {

	try {

		const response = await fetch('/world.svg');
		const text = await response.text();

		svgContent.value = text.replace(/<svg[^>]*>|<\/svg>/g, '');

		if (props.isConnected && props.tunnel) {

			const res = await getGeoLocation(props.tunnel);
			if (res && res.country) {

				flyToCountry(res.country);
			}

		};

		await placePointsOnMap();

	} catch (err) {
		console.error("Failed to load map:", err);
	};

});

watch(() => [props.tunnel, props.isConnected], async ([addr, connected]) => {

	console.log("Map Watch Triggered:", { addr, connected });

	if (addr && svgContent.value) {

		await nextTick();

		console.log("Fetching geo for:", addr);

		const res = await getGeoLocation(addr);

		if (res && res.country) {
			flyToCountry(res.country);
		};

	};

}, { immediate: true });

</script>

<template>
	<div ref="mapContainer"
		class="absolute inset-0 w-full h-full overflow-hidden bg-backgroun cursor-grab active:cursor-grabbing"
		@mousedown="startPan" @mousemove="doPan" @mouseup="stopPan" @mouseleave="stopPan" @wheel.prevent="handleWheel">

		<div class="absolute -z-10 inset-0 h-full w-full 
			bg-[linear-gradient(to_right,#73737320_1px,transparent_1px),linear-gradient(to_bottom,#73737320_1px,transparent_1px)] 
			bg-size-[16px_16px]
			mask-[radial-gradient(ellipse_50%_50%_at_50%_50%,#000_80%,transparent_100%)]" />


		<svg viewBox="0 0 2000 857" class="w-full h-full relative z-10">

			<defs>
				<radialGradient id="dot-on" cx="50%" cy="30%">
					<stop offset="0%" stop-color="#cefb04" />
					<stop offset="100%" stop-color="#a5c903" />
				</radialGradient>
				<radialGradient id="dot-off" cx="60%" cy="30%">
					<stop offset="0%" stop-color="#fb04ce" />
					<stop offset="100%" stop-color="#c903a5" />
				</radialGradient>
				<radialGradient id="dot" cx="60%" cy="30%">
					<stop offset="0%" stop-color="oklch(70.7% 0.022 261.325)" />
					<stop offset="100%" stop-color="oklch(55.1% 0.027 264.364)" />
				</radialGradient>
			</defs>

			<g :style="{
				transform: `translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})`,
				transformOrigin: '0 0',
				transition: isDragging
					? 'none'
					: 'transform 1.5s cubic-bezier(0.19, 1, 0.22, 1)',
			}">
				<g v-html="svgContent" class="fill-[#2b2c36] stroke-[#676a82]/20"></g>

				<template v-for="p in allMarkers" :key="p.name">
					<circle class="hover:cursor-default" v-if="p.name !== props.tunnel" :cx="p.x" :cy="p.y"
						@click="startTunnel(p)" r="3" fill="url(#dot)" />
				</template>

				<circle :cx="dotPos.x" :cy="dotPos.y" r="3" :fill="isConnected ? 'url(#dot-on)' : 'url(#dot-off)'" />

				<circle v-if="dotPos.x !== 0" :cx="dotPos.x" :cy="dotPos.y" r="4"
					:fill="isConnected ? 'url(#dot-on)' : 'url(#dot-off)'"
					:class="isConnected ? 'drop-shadow-[0_0_15px_rgba(16,185,129,1)]' : 'drop-shadow-[0_0_15px_oklch(63.7% 0.237 25.331)]'" />
			</g>

		</svg>
	</div>
</template>

<style scoped>
:deep(path) {
	stroke-width: 0.3px;
	vector-effect: non-scaling-stroke;
	stroke: #676a82;
}
</style>
