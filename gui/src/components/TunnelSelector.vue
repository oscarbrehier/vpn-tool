<script setup lang="ts">
import { Plus, Search, Trash2, X } from 'lucide-vue-next';
import { TunnelMetadata, UnifiedEndpoint } from '../lib/tunnel';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
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
	(e: 'delete', config: TunnelMetadata): void;
	(e: 'refresh'): void;
}>();

const query = ref<string>("");
const configurrationModal = ref(false);
const nodeSelectorRef = ref<HTMLElement | null>(null);
const newConfigModalRef = ref<HTMLElement | null>(null);

const filteredEndpoints = computed(() => {

	if (!query.value) return props.endpoints;

	const q = query.value.toLowerCase();

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
};

function handleDelete(e: MouseEvent, config: TunnelMetadata) {
	e.stopPropagation();
	emit('delete', config);
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

	<Transition name="panel">

		<div v-if="isOpen" class="h-full w-96 absolute z-100 p-4">

			<div
				class="w-full bg-neutral-500/20 backdrop-blur-2xl rounded-2xl border border-neutral-200/10 flex items-center justify-between px-4 py-3">

				<div>
					<p class="font-medium text-neutral-100">Select Server</p>
				</div>

				<div class="flex space-x-2">

					<button @click="configurrationModal = true"
						class="backdrop-blur-lg border border-neutral-200/10 h-8 px-4 rounded-full text-sm">
						<p class="pt-0.5">Add Configuration</p>
					</button>

					<button @click="$emit('close')"
						class="backdrop-blur-lg border border-neutral-200/10 size-8 rounded-full flex items-center justify-center">
						<X :size="14" />
					</button>

				</div>

			</div>

			<div class="overflow-y-auto max-h-[calc(100vh-200px)] hide-scrollbar">

				<TransitionGroup name="staggered-list" tag="div">

					<div v-for="(endpoint) in filteredEndpoints" :key="endpoint.config.name"
						class="group first:mt-3 mt-2 w-full bg-neutral-500/20 backdrop-blur-2xl rounded-2xl border border-neutral-200/10 flex items-center justify-between px-4 py-3 relative">

						<div class="flex items-center gap-x-4">

							<div class="size-10 rounded-full overflow-hidden border border-white/5">
								<img :src="`https://flagcdn.com/h80/${endpoint.geo.country_code.toLowerCase()}.webp`"
									class="w-full h-full object-cover" />
							</div>

							<div class="flex flex-col">
								<p v-if="activeTunnel === endpoint.config.public_ip"
									class="text-[10px] uppercase tracking-widest font-bold text-accent-500">
									Connected
								</p>
								<p class="text-neutral-100 text-sm font-medium">{{ endpoint.config.name }}</p>
							</div>



						</div>

						<div class="flex items-center space-x-2">

							<button @click="emit('connect', endpoint.config)"
								v-if="activeTunnel !== endpoint.config.public_ip"
								class="backdrop-blur-lg border border-neutral-200/10 h-8 px-4 rounded-full text-sm">
								<p class="pt-0.5">Connect</p>
							</button>

							<button @click="(e) => handleDelete(e, endpoint.config)"
								class="backdrop-blur-lg border border-neutral-200/10 size-8 flex items-center justify-center rounded-full text-sm">
								<Trash2 :size="14" />
							</button>

						</div>

					</div>

				</TransitionGroup>

			</div>

		</div>

	</Transition>

</template>

<style scoped>
.panel-enter-active,
.panel-leave-active {
	transition: all 0.4s cubic-bezier(0.16, 1, 0.3, 1);
}

.panel-enter-from,
.panel-leave-to {
	opacity: 0;
	transform: translateX(-20px);
}

.staggered-list-enter-active {
	transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
	transition-delay: calc(var(--i) * 0.07s);
}

.staggered-list-enter-from {
	opacity: 0;
	transform: translateY(20px) scale(0.95);
}

.staggered-list-leave-active {
	transition: all 0.3s ease;
	position: absolute;
	width: 100%;
}

.staggered-list-leave-to {
	opacity: 0;
	transform: scale(0.9);
}

.staggered-list-move {
	transition: transform 0.4s ease;
}
</style>