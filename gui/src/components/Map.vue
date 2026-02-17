<script setup lang="ts">
import { ref, reactive, onMounted, watch, nextTick } from 'vue';

const props = defineProps(['lat', 'lon', 'country']);

const svgContent = ref('');
const dotPos = reactive({ x: 0, y: 0 });
const isDragging = ref(false);

const isConnected = ref(false);
const mapContainer = ref<HTMLElement | null>(null);

const SVG_W = 2000;
const SVG_H = 857;

const transform = reactive({
	x: 0,
	y: 0,
	scale: 1
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

	const zoomSpeed = 0.1;
	const delta = e.deltaY > 0 ? -zoomSpeed : zoomSpeed;

	const oldScale = transform.scale;

	const newScale = Math.max(1, Math.min(8, oldScale + delta));

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

async function flyToCountry(countryName: string, zoomLevel = 6.5) {

	if (!countryName || !svgContent.value) return;
	await nextTick();

	const nameWithUnderscore = countryName.replace(/\s+/g, '_');
	const nameWithSpace = countryName;

	const selectors = [
		`.${nameWithUnderscore}`,
		`[id="${nameWithSpace}"]`,
		`[id="${nameWithUnderscore}"]`,
		`[name="${nameWithSpace}"]`,
		`[class*="${nameWithSpace}"]`
	];

	const elements = document.querySelectorAll(selectors.join(', '));

	if (elements.length > 0) {

		let largestArea = 0;
		let bestBBox: SVGRect | null = null;

		elements.forEach((el) => {

			if (el instanceof SVGGraphicsElement) {
				const bbox = el.getBBox();

				const area = bbox.width * bbox.height;

				if (area > largestArea) {
					largestArea = area;
					bestBBox = bbox;
				};

			};

		});

		if (bestBBox) {

			const targetX = bestBBox.x + bestBBox.width / 2;
			const targetY = bestBBox.y + bestBBox.height / 2;

			dotPos.x = targetX;
			dotPos.y = targetY;

			transform.scale = zoomLevel;

			const centerX = SVG_W / 2;
			const centerY = SVG_H / 2;

			transform.x = centerX - (targetX * zoomLevel);
			transform.y = centerY - (targetY * zoomLevel);

			const minX = SVG_W - (SVG_W * zoomLevel);
			const minY = SVG_H - (SVG_H * zoomLevel);

			transform.x = Math.min(0, Math.max(transform.x, minX));
			transform.y = Math.min(0, Math.max(transform.y, minY));

		}
	}

};

onMounted(async () => {

	try {

		const response = await fetch('/world.svg');
		const text = await response.text();

		svgContent.value = text.replace(/<svg[^>]*>|<\/svg>/g, '');

		if (props.country) {
			flyToCountry(props.country);
		};

	} catch (err) {
		console.error("Failed to load map:", err);
	};

});

watch(() => props.country, (newCountry) => {

	if (newCountry && svgContent.value) {
		console.log(newCountry);
		flyToCountry(newCountry);
	};
	
});

</script>

<template>
	<div ref="mapContainer"
		class="absolute inset-0 w-full h-full overflow-hidden bg-[#1a1b26] cursor-grab active:cursor-grabbing"
		@mousedown="startPan" @mousemove="doPan" @mouseup="stopPan" @mouseleave="stopPan" @wheel.prevent="handleWheel">

		<svg viewBox="0 0 2000 857" class="w-full h-full">
			<g :style="{
				transform: `translate(${transform.x}px, ${transform.y}px) scale(${transform.scale})`,
				transformOrigin: '0 0',
				transition: isDragging ? 'none' : 'transform 1.5s cubic-bezier(0.19, 1, 0.22, 1)'
			}">
				<g v-html="svgContent" class="fill-[#2b2c36] stroke-[#676a82]/20"></g>

				<circle v-if="dotPos.x !== 0" :cx="dotPos.x" :cy="dotPos.y" r="10" fill="#10b981"
					class="drop-shadow-[0_0_15px_rgba(16,185,129,1)]" />
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