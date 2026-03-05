<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { Upload, X } from 'lucide-vue-next';
import { ref } from 'vue';
import { toast } from 'vue-sonner';


defineProps<{ isOpen: boolean }>();
const emit = defineEmits(["close", "success"]);

const form = ref({
	name: "",
	serverIp: "",
	user: "",
	keyFile: ""
});
const isSaving = ref(false);


async function selectKeyFile() {

	const selected = await open({
		multiple: false,
		directory: false,
		filters: [{
			name: 'SSH Key',
			extensions: ['*']
		}]
	});

	if (selected && typeof selected === 'string') {
		form.value.keyFile = selected;
	};

};

async function handleSave() {

	if (!form.value.name || !form.value.serverIp || !form.value.user || !form.value.keyFile) {
		toast.error("Please fill in all fields");
		return;
	};

	isSaving.value = true;

	try {

		await invoke("setup_server", form.value);

		emit("success");
		closeSettings();

	} catch (error) {
		const message = error instanceof Error ? error.message : "Unknown error";
		toast(message);
	} finally {
		isSaving.value = false;
	};

};

const closeSettings = () => emit('close');

</script>

<template>

	<Teleport to="body">

		<Transition name="fade">

			<div v-if="isOpen"
				class="absolute top-0 left-0 z-110 flex items-center justify-center h-screen w-full bg-neutral-700/30 backdrop-blur-xs">

				<div class="h-auto w-full max-w-lg bg-neutral-800 rounded-lg p-4 border">

					<div class="w-full flex items-center justify-between">

						<div class="flex flex-col">
							<p class="capitalize">add new gateway</p>
							<p class="text-sm text-neutral-400">Enter your server details to expand your secure network
							</p>
						</div>

						<button @click="closeSettings" class="p-1 rounded-full bg-neutral-700 border">
							<X :size="16" />
						</button>

					</div>

					<div class="mt-8 space-y-4">

						<div class="grid grid-cols-2 gap-4">

							<div className="flex flex-col gap-1.5">
								<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
									Name
								</label>
								<input v-model="form.name" id="name" placeholder="e.g. Prod. Server"
									class="bg-neutral-700 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between transition-colors outline-none">
							</div>

							<div className="flex flex-col gap-1.5">
								<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
									SSH Username
								</label>
								<input v-model="form.user" id="ssh-user" placeholder="e.g. root"
									class="bg-neutral-700 border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between transition-colors outline-none">
							</div>


						</div>

						<div className="flex flex-col gap-1.5">
							<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
								IP Address
							</label>
							<input v-model="form.serverIp" id="host" placeholder="0.0.0.0"
								class="bg-neutral-700 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between transition-colors outline-none">
						</div>

						<div class="flex flex-col gap-2">
							<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
								SSH Private Key
							</label>

							<button @click="selectKeyFile"
								class="bg-neutral-700 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between transition-colors">
								<div class="flex items-center gap-4 overflow-hidden">
									<Upload class="size-3 shrink-0" />
									<span class="truncate text-neutral-300 text-sm pr-4">
										{{ form.keyFile ? form.keyFile : 'Select Private Key' }}
									</span>
								</div>
								<span v-if="form.keyFile"
									class="text-[10px] text-emerald-500 font-bold uppercase">Selected</span>
							</button>
						</div>

					</div>

					<button @click="handleSave" :disabled="isSaving"
						class="mt-8 bg-accent-600 text-black w-full py-3 rounded-md">
						{{ isSaving ? 'Adding...' : 'Add configuration' }}
					</button>

				</div>

			</div>

		</Transition>

	</Teleport>

</template>