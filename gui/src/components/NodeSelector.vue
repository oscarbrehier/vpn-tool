<script setup lang="ts">
import { Plus, Search, X } from 'lucide-vue-next';
import { UnifiedEndpoint } from '../App.vue';
import { TunnelMetadata } from '../lib/tunnel';
import { computed, onUnmounted, ref, watch } from 'vue';
import NewConfigurationModal from './NewConfigurationModal.vue';

const props = defineProps<{
	isOpen: boolean;
	endpoints: UnifiedEndpoint[];
	activeTunnel: string | null;
	isPending: boolean;
}>();

const emit = defineEmits<{
	(e: 'close'): void;
	(e: 'connect', config: TunnelMetadata): void;
	(e: 'refresh'): void;
}>();

const query = ref<string>("");
const configurrationModal = ref(false);
const nodeSelectorRef = ref<HTMLElement | null>(null);
const newConfigModalRef = ref<HTMLElement | null>(null);

const filteredEndpoints = computed(() => {

	if (!query.value) return props.endpoints;

	const q = query.value.toLowerCase();

	console.log(q)
	console.log(props.endpoints)

	return props.endpoints.filter(
		i => i.config.name.toLowerCase().includes(q)
	);

});

function handleOutsideClick(e: MouseEvent) {

	const target = e.target as Node;
	const isInsideSelector = nodeSelectorRef.value?.contains(target);
	const isInsideConfigModal = newConfigModalRef.value?.contains(target);

	if (!isInsideSelector && !isInsideConfigModal) {
		emit('close');
	};

};

function handleConfigSuccess() {
	configurrationModal.value = false;
	emit("refresh");
}

watch(() => props.isOpen, (isOpen) => {

	if (isOpen) {
		setTimeout(() => document.addEventListener("click", handleOutsideClick), 0);
	} else {
		document.removeEventListener("click", handleOutsideClick);
	};

});

onUnmounted(() => {
	document.removeEventListener("click", handleOutsideClick);
});

</script>

<template>

	<NewConfigurationModal :is-open="configurrationModal" @close="configurrationModal = false"
		@success="handleConfigSuccess" ref="newConfigModalRef" />

	<Transition name="slide">

		<div v-if="isOpen" ref="nodeSelectorRef" class="absolute z-100 w-96 h-full pointer-events-none">

			<div class="absolute inset-4 pointer-events-auto">

				<div class="w-full h-full bg-neutral-800/50 backdrop-blur-sm rounded-lg p-4 border border-neutral-400/10">

					<div class="w-full flex items-start justify-between gap-x-8">

						<div>
							<p class="font-medium text-lg text-neutral-100">Server list</p>
							<p class="font-medium text-sm text-neutral-400">Select a server to continue with
								a
								secure connection</p>
						</div>

						<button @click="emit('close')" class="rounded-full bg-neutral-700 p-1.5">
							<X class="text-neutral-100" :size="18" />
						</button>

					</div>

					<div class="w-full flex items-center justify-between space-x-2 mt-6">

						<div class="h-10 flex-1 rounded-md bg-neutral-700/40 flex items-center px-4 space-x-4">
							<Search class="text-neutral-400" :size="20" />
							<input v-model="query" placeholder="Search..." class="outline-none" />
						</div>

						<button @click="configurrationModal = true"
							class="size-10 bg-neutral-700/40 rounded-md text-neutral-300 flex items-center justify-center hover:bg-neutral-700/80">
							<Plus />
						</button>

					</div>

					<div class="mt-10 overflow-y-auto max-h-[calc(100vh-250px)] hide-scrollbar">

						<TransitionGroup name="stagger" appear>

							<button v-for="(endpoint, index) in filteredEndpoints" :key="endpoint.config.name"
								@click="emit('connect', endpoint.config)"
								:style="{ transitionDelay: `${index * 50}ms` }"
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

									<p class="text-neutral-100 text-sm font-medium">{{ endpoint.config.name }}</p>

								</div>

								<div v-if="activeTunnel === endpoint.config.public_ip"
									class="flex items-center gap-x-4">

									<span class="text-[10px] uppercase tracking-widest font-bold text-accent-500">
										Connected
									</span>

									<div
										class="size-2 rounded-full bg-accent-500 shadow-[0_0_8px_var(--color-accent-500)] animate-pulse" />

								</div>

							</button>

						</TransitionGroup>

					</div>

				</div>

			</div>

		</div>

	</Transition>

</template>