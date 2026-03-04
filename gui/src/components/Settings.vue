<script setup lang="ts">

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { EthernetPort, Upload, X } from "lucide-vue-next";
import { ref } from "vue";
import { toast } from "vue-sonner";

defineProps<{ isOpen: boolean }>();
const emit = defineEmits(['close']);

const hostIp = ref("");
const sshUser = ref("");
const sshPath = ref("");
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
		sshPath.value = selected;
	};

};

async function handleSave() {

	if (!hostIp.value || !sshUser.value || !sshPath.value) {
		alert("Please fill in all fields");
		return;
	}

	isSaving.value = true;

	try {

		await invoke("setup_server", {
			serverIp: hostIp.value,
			user: sshUser.value,
			keyFile: sshPath.value
		});

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

			<div v-if="isOpen" class="fixed inset-0 w-full h-full bg-neutral-700/40 z-50 backdrop-blur-xl pt-16 p-8 flex flex-col">

				<div class="w-full flex items-center justify-between mb-6">

					<p class="text-3xl font-extralight">Settings</p>

					<button @click="closeSettings" class="rounded-full bg-neutral-600 p-1.5">
						<X class="text-neutral-100" :size="18" />
					</button>

				</div>

				<div class="h-full w-full grid grid-cols-3 gap-4">

					<div class="h-full w-full">

						<butotn class="flex items-center space-x-3 bg-neutral-800/40 py-2 px-4 rounded-md border cursor-pointer">
							<EthernetPort :size="18" />
							<p>Configuration</p>
						</butotn>

					</div>

					<div class="col-span-2 w-full h-full bg-neutral-800/40 border backdrop-blur-xl rounded-xl p-4">
					
						<div class="w-full">

							<div class="mb-6">
								<p class="capitalize">new tunnel</p>
								<p class="text-sm text-neutral-400">Configure a new SSH tunnel to your server</p>
							</div>

							<div class="space-y-6">

								<div className="flex flex-col gap-1.5">
									<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
										IP Address
									</label>
									<input v-model="hostIp" id="host" placeholder="e.g. "
										class="bg-neutral-800/50 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between hover:bg-neutral-800 transition-colors">
								</div>

								<div className="flex flex-col gap-1.5">
									<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
										SSH Username
									</label>
									<input v-model="sshUser" id="ssh-user" placeholder="e.g. root"
										class="bg-neutral-800/50 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between hover:bg-neutral-800 transition-colors">
								</div>

								<div class="flex flex-col gap-2">
									<label class="text-xs font-semibold text-neutral-400 uppercase tracking-wider">
										SSH Private Key
									</label>

									<button @click="selectKeyFile"
										class="bg-neutral-800/50 border border-white/5 w-full rounded-xl px-4 py-3 text-sm font-mono flex items-center justify-between hover:bg-neutral-800 transition-colors">
										<div class="flex items-center gap-4 overflow-hidden">
											<Upload class="size-3 shrink-0" />
											<span class="truncate text-neutral-300 text-sm pr-4">
												{{ sshPath ? sshPath : 'Select Private Key' }}
											</span>
										</div>
										<span v-if="sshPath"
											class="text-[10px] text-emerald-500 font-bold uppercase">Selected</span>
									</button>
								</div>

							</div>

							<button @click="handleSave" :disabled="isSaving"
								class="px-4 py-2 bg-accent-700 mt-8 text-black text-sm font-semibold">
								{{ isSaving ? 'Saving...' : 'Add configuration' }}
							</button>

						</div>
					</div>

				</div>



			</div>

		</Transition>

	</Teleport>

</template>

<style scoped>
.fade-enter-active,
.fade-leave-active {
	transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
	opacity: 0;
}
</style>