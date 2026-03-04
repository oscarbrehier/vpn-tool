<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { Minus, X } from 'lucide-vue-next';

const minimize = async () => await getCurrentWindow().minimize();
const exit = async () => await getCurrentWindow().close();

</script>

<template>
	<Teleport to="body">

		<div data-tauri-drag-region class="absolute top-0 left-0 w-full h-14 z-55 cursor-default titlebar"></div>

		<div class="h-auto w-auto absolute top-0 right-0 z-60 p-4 space-x-4 flex items-center justify-end select-none">

			<button @click="minimize">
				<Minus />
			</button>

			<button @click="exit">
				<X />
			</button>

		</div>

	</Teleport>
</template>