import { runCommand } from "./tauri";

export async function startPinging() {
	await runCommand("start_ping_loop");
};

export async function stopPinging() {
	await runCommand("stop_ping_loop");
};