use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;
use sysinfo::{System};

#[derive(Debug, Serialize, Clone)]
pub struct AppGroup {
    pub pids: Vec<u32>,
    pub name: String,
    pub path: Option<PathBuf>,
    pub icon_base64: Option<String>,
}

pub fn get_running_apps() -> Result<Vec<AppGroup>, String> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut grouped_processes: HashMap<String, AppGroup> = HashMap::new();

    for (pid, process) in sys.processes() {
        if let Some(path) = process.exe() {
            let path_str = path.to_string_lossy();

            let is_user_app = path_str.contains("Program Files")
                || path_str.contains("AppData")
                || path_str.contains("Users");

            if is_user_app {
                let name = process.name().to_string_lossy().replace(".exe", "");

                let entry = grouped_processes.entry(name.clone()).or_insert(AppGroup {
                    name: name,
                    path: Some(path.to_path_buf()),
                    icon_base64: None,
                    pids: Vec::new(),
                });

                entry.pids.push(pid.as_u32());
            }
        }
    }

    Ok(grouped_processes.values().cloned().collect())
}
