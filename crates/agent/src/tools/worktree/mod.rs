use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeState {
    pub path: PathBuf,
    pub name: String,
    pub is_primary: bool,
    pub created_at: u64,
}

pub struct WorktreeManager {
    worktrees: Arc<RwLock<HashMap<String, WorktreeState>>>,
}

impl Default for WorktreeManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorktreeManager {
    pub fn new() -> Self {
        Self {
            worktrees: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, name: String, path: PathBuf, is_primary: bool) -> WorktreeState {
        let state = WorktreeState {
            path,
            name: name.clone(),
            is_primary,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        let mut worktrees = self.worktrees.write().await;
        worktrees.insert(name, state.clone());
        state
    }

    pub async fn remove(&self, name: &str) -> bool {
        let mut worktrees = self.worktrees.write().await;
        worktrees.remove(name).is_some()
    }

    pub async fn get(&self, name: &str) -> Option<WorktreeState> {
        let worktrees = self.worktrees.read().await;
        worktrees.get(name).cloned()
    }

    pub async fn list(&self) -> Vec<WorktreeState> {
        let worktrees = self.worktrees.read().await;
        worktrees.values().cloned().collect()
    }

    pub async fn get_primary(&self) -> Option<WorktreeState> {
        let worktrees = self.worktrees.read().await;
        worktrees.values().find(|w| w.is_primary).cloned()
    }
}
