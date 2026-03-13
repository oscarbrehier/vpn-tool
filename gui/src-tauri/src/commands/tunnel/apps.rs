// #[cfg(target_os = "windows")]
// use vpn_lib::app_filter::scanner::get_windows_icon;
// use vpn_lib::app_filter::scanner::{AppGroup, get_running_apps};

// use crate::AppCache;
// use crate::commands::tunnel::RedirectionState;

// #[tauri::command]
// pub async fn update_tunneled_apps(state: tauri::State<'_, RedirectionState>, pids: Vec<u32>) -> Result<(), String> {

// 	let guard = state.filter_tx.lock().await;

// 	println!("update_tunnel_apps: received new pids {:?}", pids);

// 	if let Some(tx) = &*guard {
// 		tx.send(pids.clone()).map_err(|e| e.to_string())?;

// 		drop(guard);

// 		let mut pids_guard = state.tunneled_pids.lock().await;
// 		*pids_guard = pids;

// 		Ok(())
// 	} else {
// 		println!("No active redirection loop");
// 		Err("No active redirection loop".into())
// 	}

// }

// #[tauri::command]
// pub async fn fetch_apps(
// 	cache: tauri::State<'_, AppCache>
// ) -> Result<Vec<AppGroup>, String> {

// 	get_running_apps(|path| {

// 		let path_str = path.to_string_lossy().to_string();

// 		if let Some(icon) = cache.icons.get(&path_str) {
// 			return Some(icon.value().clone());
// 		}

// 		#[cfg(target_os = "windows")]
// 		if let Some(new_icon) = get_windows_icon(path) {
// 			cache.icons.insert(path_str, new_icon.clone());
// 			return Some(new_icon);
// 		}

// 		None

// 	})

// }

// #[tauri::command]
// pub async fn get_tunneled_apps(state: tauri::State<'_, RedirectionState>) -> Result<Vec<u32>, String> {
	
// 	let pids = state.tunneled_pids.lock().await;
// 	Ok(pids.clone())

// }