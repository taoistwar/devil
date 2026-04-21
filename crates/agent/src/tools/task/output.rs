use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutputInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutputOutput {
    pub task_id: String,
    pub output: Option<String>,
    pub is_complete: bool,
}

pub struct TaskOutputStore {
    outputs: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for TaskOutputStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskOutputStore {
    pub fn new() -> Self {
        Self {
            outputs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn store_output(&self, id: &str, output: String) {
        let mut outputs = self.outputs.write().await;
        outputs.insert(id.to_string(), output);
    }

    pub async fn get_output(&self, id: &str) -> Option<String> {
        let outputs = self.outputs.read().await;
        outputs.get(id).cloned()
    }
}

pub struct TaskOutputTool {
    store: TaskOutputStore,
}

impl TaskOutputTool {
    pub fn new(store: TaskOutputStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for TaskOutputTool {
    type Input = TaskOutputInput;
    type Output = TaskOutputOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_output"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ID of the task to get output from"
                }
            },
            "required": ["id"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
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
        let output = self.store.get_output(&input.id).await;

        let result_output = TaskOutputOutput {
            task_id: input.id,
            output,
            is_complete: true,
        };

        Ok(ToolResult::success("task_output-1", result_output))
    }
}
