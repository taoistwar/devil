use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentState {
    Idle,
    Planning,
    Working,
    WaitingForPermission,
}

impl Default for AgentState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanMode {
    Browse,
    Task,
    Debug,
}

impl Default for PlanMode {
    fn default() -> Self {
        Self::Task
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningContext {
    pub mode: PlanMode,
    pub active_tasks: Vec<String>,
    pub completed_tasks: Vec<String>,
    pub notes: Vec<String>,
}

impl Default for PlanningContext {
    fn default() -> Self {
        Self {
            mode: PlanMode::Task,
            active_tasks: Vec::new(),
            completed_tasks: Vec::new(),
            notes: Vec::new(),
        }
    }
}
