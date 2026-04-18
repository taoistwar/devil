use crate::tools::planning::state::AgentState;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TaskScheduler {
    state: Arc<RwLock<AgentState>>,
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskScheduler {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AgentState::default())),
        }
    }

    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }

    pub async fn set_state(&self, new_state: AgentState) {
        *self.state.write().await = new_state;
    }
}
