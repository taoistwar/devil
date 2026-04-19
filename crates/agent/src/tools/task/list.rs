use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::task::{TaskListOptions, TaskStatus, TaskStore};
use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListInput {
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListOutput {
    pub tasks: Vec<TaskSummary>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

pub struct TaskListTool {
    store: TaskStore,
}

impl TaskListTool {
    pub fn new(store: TaskStore) -> Self {
        Self { store }
    }
}

fn parse_status(s: &str) -> Option<TaskStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(TaskStatus::Pending),
        "in_progress" | "inprogress" => Some(TaskStatus::InProgress),
        "completed" => Some(TaskStatus::Completed),
        "cancelled" => Some(TaskStatus::Cancelled),
        _ => None,
    }
}

#[async_trait]
impl Tool for TaskListTool {
    type Input = TaskListInput;
    type Output = TaskListOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_list"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "cancelled"],
                    "description": "Filter by status"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Filter by tags"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of tasks to return"
                }
            }
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
        let status = input.status.as_ref().and_then(|s| parse_status(s));

        let options = TaskListOptions {
            status,
            tags: input.tags,
            limit: input.limit,
        };

        let tasks = self.store.list(options).await;

        let output = TaskListOutput {
            tasks: tasks
                .iter()
                .map(|t| TaskSummary {
                    id: t.id.clone(),
                    title: t.title.clone(),
                    status: format!("{:?}", t.status).to_lowercase(),
                    priority: format!("{:?}", t.priority).to_lowercase(),
                })
                .collect(),
            total: tasks.len(),
        };

        Ok(ToolResult::success("task_list-1", output))
    }
}
