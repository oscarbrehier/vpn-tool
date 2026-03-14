import { invoke } from "@tauri-apps/api/core";
import { toast } from "vue-sonner";

type CommandResult<T> =
	| { data: T; error: null }
	| { data: null; error: string };

export async function runCommand<T>(cmd: string, notifyError: boolean = false, args: Record<string, any> = {}): Promise<CommandResult<T>> {

	try {

		const data = await invoke<T>(cmd, args);
		return { data, error: null };

	} catch (err) {

		const message = typeof err === "string" ? err : JSON.stringify(err);

		if (notifyError) toast(message);

		console.log(message);
		return { data: null, error: message };

	};

};