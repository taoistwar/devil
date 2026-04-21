use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tools::task::TaskStore;
use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStopInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStopOutput {
    pub task_id: String,
    pub success: bool,
}

pub struct TaskStopTool {
    #[allow(dead_code)]
    store: TaskStore,
    running_tasks: Arc<RwLock<std::collections::HashSet<String>>>,
}

impl TaskStopTool {
    pub fn new(store: TaskStore) -> Self {
        Self {
            store,
            running_tasks: Arc::new(RwLock::new(std::collections::HashSet::new())),
        }
    }

    pub async fn mark_running(&self, id: &str) {
        let mut tasks = self.running_tasks.write().await;
        tasks.insert(id.to_string());
    }

    pub async fn is_running(&self, id: &str) -> bool {
        let tasks = self.running_tasks.read().await;
        tasks.contains(id)
    }
}

#[async_trait]
impl Tool for TaskStopTool {
    type Input = TaskStopInput;
    type Output = TaskStopOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_stop"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ID of the task to stop"
                }
            },
            "required": ["id"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let mut running = self.running_tasks.write().await;
        running.remove(&input.id);

        let output = TaskStopOutput {
            task_id: input.id,
            success: true,
        };

        Ok(ToolResult::success("task_stop-1", output))
    }
}
