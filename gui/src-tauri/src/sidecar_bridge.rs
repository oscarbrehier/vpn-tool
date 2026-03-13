use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize)]
struct SidecarResponse {
	success: bool,
	message: String,
	data: Option<serde_json::Value>
}

async fn run_sidecar_command(app: tauri::AppHandle, args: Vec<&str>) -> Result<SidecarResponse> {

	let sidecar_command = app.sh

}