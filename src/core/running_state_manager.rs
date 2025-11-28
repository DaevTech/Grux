use crate::core::{running_state::RunningState};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;

pub struct RunningStateManager {
    pub current_running_state: Arc<RwLock<RunningState>>,
}

impl RunningStateManager {
    pub fn new() -> Self {
        let current_running_state = Arc::new(RwLock::new(RunningState::new()));
        RunningStateManager { current_running_state }
    }

    pub fn get_running_state(&self) -> Arc<RwLock<RunningState>> {
        self.current_running_state.clone()
    }

    pub async fn set_new_running_state(&self) {
        let triggers = crate::core::triggers::get_trigger_handler();

        // Optain a write lock to update the running state
        let mut current_state = self.current_running_state.write().await;
        // cancel current token to notify any tasks depending on it
        triggers.run_trigger("stop_services").await;
        // Give a small delay to allow tasks to notice cancellation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        // Setup a new running state
        *current_state = RunningState::new();
    }
}

static RUNNING_STATE_MANAGER_SINGLETON: OnceLock<RunningStateManager> = OnceLock::new();

pub fn get_running_state_manager() -> &'static RunningStateManager {
    RUNNING_STATE_MANAGER_SINGLETON.get_or_init(|| RunningStateManager::new())
}
