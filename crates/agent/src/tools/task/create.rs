use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::task::{Task, TaskPriority, TaskStore};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateOutput {
    pub task: TaskInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub title: String,
    pub status: String,
    pub priority: String,
}

pub struct TaskCreateTool {
    store: TaskStore,
}

impl TaskCreateTool {
    pub fn new(store: TaskStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for TaskCreateTool {
    type Input = TaskCreateInput;
    type Output = TaskCreateOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "task_create"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Title of the task"
                },
                "description": {
                    "type": "string",
                    "description": "Detailed description of the task"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "medium", "high", "urgent"],
                    "description": "Priority level of the task"
                },
                "tags": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Tags to associate with the task"
                }
            },
            "required": ["title"]
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
        let mut task = Task::new(input.title);
        
        if let Some(desc) = input.description {
            task.description = Some(desc);
        }
        
        if let Some(priority) = input.priority {
            task.priority = priority;
        }
        
        if let Some(tags) = input.tags {
            task.tags = tags;
        }

        let created = self.store.create(task).await;

        let output = TaskCreateOutput {
            task: TaskInfo {
                id: created.id,
                title: created.title,
                status: format!("{:?}", created.status).to_lowercase(),
                priority: format!("{:?}", created.priority).to_lowercase(),
            },
        };

        Ok(ToolResult::success("task_create-1", output))
    }
}
