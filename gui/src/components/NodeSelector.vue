<script setup lang="ts">
import { Search, X } from 'lucide-vue-next';
import { UnifiedEndpoint } from '../App.vue';
import { TunnelMetadata } from '../lib/tunnel';


defineProps<{
	isOpen: boolean;
	endpoints: UnifiedEndpoint[];
	activeTunnel: string | null;
	isPending: boolean;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'connect', config: TunnelMetadata): void;
}>();

</script>

<template>

	<Transition name="slide">
		<div v-if="isOpen" class="absolute z-80 w-96 h-full p-4">

			<div
				class="w-full h-full bg-neutral-800/50 rounded-lg backdrop-blur-sm p-4 border border-neutral-400/10 select-none">

				<div class="w-full flex items-start justify-between gap-x-8">

					<div>
						<p class="font-medium text-lg select-none text-neutral-100">Server list</p>
						<p class="font-medium text-sm select-none text-neutral-400">Select a server to continue with a secure connection</p>
					</div>

					<button @click="emit('close')" class="rounded-full bg-neutral-700 p-1.5">
						<X class="text-neutral-100" :size="18" />
					</button>

				</div>

				<div class="h-10 w-full rounded-md bg-neutral-700 mt-6 flex items-center px-4 space-x-4">
					<Search class="text-neutral-400" :size="20" />
					<input placeholder="Search..." class="outline-none" />
				</div>

				<div class="mt-10 overflow-y-auto max-h-[calc(100vh-250px)] pr-2">
					<TransitionGroup name="stagger" appear>
						<button v-for="(endpoint, index) in endpoints" :key="endpoint.config.name"
							@click="emit('connect', endpoint.config)" :style="{ transitionDelay: `${index * 50}ms` }"
							:disabled="activeTunnel === endpoint.config.public_ip"
							class="w-full flex items-center justify-between group p-3 rounded-md mb-2 transition-all border"
							:class="activeTunnel === endpoint.config.name
								? 'bg-accent-500/20 border-accent-500/30 cursor-default'
								: 'bg-neutral-700/40 hover:bg-neutral-700/80 border-transparent cursor-pointer'">

							<div class="flex items-center gap-x-4">
								<div class="w-10 rounded-sm overflow-hidden border border-white/5">
									<img :src="`https://flagcdn.com/h80/${endpoint.geo.country_code.toLowerCase()}.webp`"
										class="w-full h-full object-cover" />
								</div>
								<p class="text-neutral-100 text-sm font-medium">{{ endpoint.geo.country }}</p>
							</div>

							<div v-if="activeTunnel === endpoint.config.public_ip" class="flex items-center gap-x-4">
								<span
									class="text-[10px] uppercase tracking-widest font-bold text-accent-500">Connected</span>
								<div
									class="size-2 rounded-full bg-accent-500 shadow-[0_0_8px_var(--color-accent-500)] animate-pulse" />
							</div>
						</button>
					</TransitionGroup>
				</div>

			</div>

		</div>
	</Transition>

</template>