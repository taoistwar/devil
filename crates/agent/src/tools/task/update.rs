use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::task::{TaskStore, TaskUpdate};
use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdateInput {
    pub id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdateOutput {
    pub task_id: String,
    pub success: bool,
}

pub struct TaskUpdateTool {
    store: TaskStore,
}

impl TaskUpdateTool {
    pub fn new(store: TaskStore) -> Self {
        Self { store }
    }
}

fn parse_status(s: &str) -> Option<crate::tools::task::TaskStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(crate::tools::task::TaskStatus::Pending),
        "in_progress" | "inprogress" => Some(crate::tools::task::TaskStatus::InProgress),
        "completed" => Some(crate::tools::task::TaskStatus::Completed),
        "cancelled" => Some(crate::tools::task::TaskStatus::Cancelled),
        _ => None,
    }
}

fn parse_priority(s: &str) -> Option<crate::tools::task::TaskPriority> {
    match s.to_lowercase().as_str() {
        "low" => Some(crate::tools::task::TaskPriority::Low),
        "medium" => Some(crate::tools::task::TaskPriority::Medium),
        "high" => Some(crate::tools::task::TaskPriority::High),
        "urgent" => Some(crate::tools::task::TaskPriority::Urgent),
        _ => None,
    }
}

#[async_trait]
impl Tool for TaskUpdateTool {
    type Input = TaskUpdateInput;
    type Output = TaskUpdateOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_update"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ID of the task to update"
                },
                "title": {
                    "type": "string",
                    "description": "New title for the task"
                },
                "description": {
                    "type": "string",
                    "description": "New description for the task"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "cancelled"],
                    "description": "New status for the task"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "urgent"],
                    "description": "New priority for the task"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "New tags for the task"
                }
            },
            "required": ["id"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
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
        let status = input.status.as_ref().and_then(|s| parse_status(s));
        let priority = input.priority.as_ref().and_then(|p| parse_priority(p));

        let update = TaskUpdate {
            title: input.title,
            description: input.description,
            status,
            priority,
            tags: input.tags,
        };

        let result = self.store.update(&input.id, update).await;

        let output = TaskUpdateOutput {
            task_id: input.id,
            success: result.is_some(),
        };

        Ok(ToolResult::success("task_update-1", output))
    }
}
