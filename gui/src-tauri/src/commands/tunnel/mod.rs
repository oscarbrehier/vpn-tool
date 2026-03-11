pub mod metadata;
pub mod tunnel;
pub mod apps;

use tokio::sync::{Mutex, mpsc};
pub use tunnel::*;

pub struct RedirectionState {
	pub tunneled_pids: Mutex<Vec<u32>>,
	pub filter_tx: Mutex<Option<mpsc::UnboundedSender<Vec<u32>>>>
}

impl Default for RedirectionState {
	fn default() -> Self {
		Self { tunneled_pids: Mutex::new(Vec::new()), filter_tx: Mutex::new(None) }
	}
}