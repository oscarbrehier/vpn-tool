<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import "./assets/globals.css";
import PowerButton from "./components/PowerButton.vue";

const greetMsg = ref("");
const name = ref("");

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
}
</script>

<template>

	<main class="h-screen w-screen flex flex-col items-center justify-center space-y-8 bg-[hsl(220,20%,6%)]">

		<div class="flex flex-col items-center gap-1">
			<div class="flex items-center gap-2">
				<div class="h-2.5 w-2.5 rounded-full transition-colors duration-300" :class="isConnected
					? 'bg-primary shadow-[0_0_8px_hsl(155,60%,50%)]'
					: 'bg-muted-foreground'" aria-hidden="true" />
				<span class="text-sm font-medium transition-colors duration-300"
					:class="isConnected ? 'text-primary' : 'text-muted-foreground'">
					{{ isConnected ? 'Connected' : 'Disconnected' }}
				</span>
			</div>
			<span class="font-mono text-4xl font-bold tracking-tight text-foreground">
				00:18:33
			</span>
		</div>

		<PowerButton :is-connected="isConnected" :is-connecting="isConnecting" @toggle="handleToggle" />

	</main>

</template>