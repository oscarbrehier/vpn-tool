use serde::{Serialize, Deserialize};
use tauri_plugin_shell::{ShellExt, process::CommandEvent};

#[derive(Deserialize, Serialize)]
pub struct SidecarResponse {
	pub success: bool,
	pub message: String,
	pub data: Option<serde_json::Value>
}

pub async fn run_sidecar_command(app: tauri::AppHandle, args: Vec<&str>) -> Result<SidecarResponse, String> {

	let sidecar_command = app.shell().sidecar("sidecar").map_err(|e| format!("Failed to create sidecar command: {}", e))?;

	let (mut rx, _child) = sidecar_command.args(args).spawn().map_err(|e| format!("Failed to spawn sidecar: {}", e))?;

	let mut output = String::new();

	while let Some(event) = rx.recv().await {
		match event {
			CommandEvent::Stdout(line) => {
				let line_str = String::from_utf8_lossy(&line);
				output.push_str(&line_str)
			},
			CommandEvent::Stderr(line) => {
				let line_str = String::from_utf8_lossy(&line);
                eprintln!("Sidecar stderr: {}", line_str);
			},
			CommandEvent::Error(err) => return Err(err),
			CommandEvent::Terminated(_) => break,
			_ => {}
		}
	}

	serde_json::from_str(&output).map_err(|e| format!("Failed to parse sidecar response: {}\n output: {}\n", e, output))

}