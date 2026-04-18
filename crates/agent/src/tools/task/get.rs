use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::task::TaskStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGetInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGetOutput {
    pub task: Option<TaskDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

pub struct TaskGetTool {
    store: TaskStore,
}

impl TaskGetTool {
    pub fn new(store: TaskStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for TaskGetTool {
    type Input = TaskGetInput;
    type Output = TaskGetOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_get"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ID of the task to retrieve"
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
        let task = self.store.get(&input.id).await;

        let output = TaskGetOutput {
            task: task.map(|t| TaskDetail {
                id: t.id,
                title: t.title,
                description: t.description,
                status: format!("{:?}", t.status).to_lowercase(),
                priority: format!("{:?}", t.priority).to_lowercase(),
                created_at: t.created_at,
                updated_at: t.updated_at,
                tags: t.tags,
            }),
        };

        Ok(ToolResult::success("task_get-1", output))
    }
}
