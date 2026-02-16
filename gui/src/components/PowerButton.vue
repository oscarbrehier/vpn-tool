<script setup lang="ts">
import { computed } from 'vue';
import { Power } from 'lucide-vue-next';

interface Props {
	isConnected: boolean
	isConnecting: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
	toggle: []
}>()

const isActive = computed(() => props.isConnected || props.isConnecting)
</script>

<template>

	<div class="relative flex h-64 w-64 items-center justify-center">

		<div class="absolute h-56 w-56 rounded-full transition-all duration-700" :class="isActive
			? 'shadow-[0_0_60px_rgba(52,211,153,0.15)]'
			: 'shadow-[0_0_40px_rgba(0,0,0,0.3)]'" />

		<svg class="absolute h-64 w-64" viewBox="0 0 256 256" fill="none" aria-hidden="true">

			<circle cx="128" cy="128" r="120" stroke="hsl(220, 15%, 18%)" stroke-width="2" />

			<circle cx="128" cy="128" r="108" stroke="hsl(220, 15%, 15%)" stroke-width="1.5" />

			<circle cx="128" cy="128" r="114" :stroke="isActive ? 'hsl(155, 60%, 50%)' : 'hsl(220, 15%, 22%)'"
				stroke-width="2" stroke-dasharray="120 600" stroke-linecap="round"
				class="origin-center transition-all duration-700" :class="isConnecting ? 'animate-spin' : ''"
				style="animation-duration: 3s" />

			<circle cx="128" cy="128" r="114" :stroke="isActive ? 'hsl(155, 60%, 40%)' : 'hsl(220, 15%, 20%)'"
				stroke-width="1.5" stroke-dasharray="80 640" stroke-dashoffset="200" stroke-linecap="round"
				class="origin-center transition-all duration-700" :class="isConnecting ? 'animate-spin' : ''"
				style="animation-duration: 4s; animation-direction: reverse" />

			<circle cx="128" cy="14" r="3" :fill="isActive ? 'hsl(40, 80%, 55%)' : 'hsl(220, 15%, 25%)'"
				class="origin-center transition-all duration-700" :class="isConnecting ? 'animate-spin' : ''"
				style="transform-origin: 128px 128px; animation-duration: 6s" />
		</svg>

		<div class="absolute h-44 w-44 rounded-full transition-all duration-500" :class="isActive
			? 'bg-linear-to-b from-[hsl(155,60%,15%)] to-[hsl(220,18%,11%)]'
			: 'bg-linear-to-b from-[hsl(220,15%,14%)] to-[hsl(220,18%,10%)]'" />

		<button
			class="relative z-10 flex h-36 w-36 items-center justify-center rounded-full transition-all duration-500 focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 focus-visible:ring-offset-background"
			:class="[
				isActive
					? 'bg-linear-to-b from-[hsl(155,55%,55%)] to-[hsl(155,60%,40%)] shadow-[0_8px_32px_rgba(52,211,153,0.35),inset_0_2px_4px_rgba(255,255,255,0.15)]'
					: 'bg-linear-to-b from-[hsl(220,15%,22%)] to-[hsl(220,15%,16%)] shadow-[0_8px_24px_rgba(0,0,0,0.4),inset_0_1px_2px_rgba(255,255,255,0.05)] hover:from-[hsl(220,15%,25%)] hover:to-[hsl(220,15%,18%)]',
				isConnecting ? 'cursor-wait' : 'cursor-pointer',
			]" :disabled="isConnecting" :aria-label="isConnected ? 'Disconnect VPN' : 'Connect VPN'" @click="emit('toggle')">

			<div class="absolute inset-4 rounded-full transition-opacity duration-500"
				:class="isActive ? 'opacity-15' : 'opacity-0'" :style="{
					backgroundImage: `linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px),
            linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)`,
					backgroundSize: '12px 12px',
				}" aria-hidden="true" />

			<Power class="relative z-10 h-12 w-12 transition-colors duration-500"
				:class="isActive ? 'text-[hsl(220,20%,10%)]' : 'text-muted-foreground'" :stroke-width="2.5" />

		</button>

	</div>

</template>