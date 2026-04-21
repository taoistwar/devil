pub mod create;
pub mod get;
pub mod list;
pub mod output;
pub mod scheduler;
pub mod stop;
pub mod update;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use create::{TaskCreateInput, TaskCreateOutput, TaskCreateTool};
pub use get::{TaskGetInput, TaskGetOutput, TaskGetTool};
pub use list::{TaskListInput, TaskListOutput, TaskListTool};
pub use output::{TaskOutputInput, TaskOutputOutput, TaskOutputStore, TaskOutputTool};
pub use scheduler::TaskScheduler;
pub use stop::{TaskStopInput, TaskStopOutput, TaskStopTool};
pub use update::{TaskUpdateInput, TaskUpdateOutput, TaskUpdateTool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: u64,
    pub updated_at: u64,
    pub tags: Vec<String>,
}

impl Task {
    pub fn new(title: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description: None,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            created_at: now,
            updated_at: now,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Low,
    #[default]
    Medium,
    High,
    Urgent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListOptions {
    pub status: Option<TaskStatus>,
    pub tags: Option<Vec<String>>,
    pub limit: Option<usize>,
}

pub struct TaskStore {
    tasks: Arc<RwLock<HashMap<String, Task>>>,
}

impl Default for TaskStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskStore {
    pub fn new() -> Self {
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, task: Task) -> Task {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id.clone(), task.clone());
        task
    }

    pub async fn get(&self, id: &str) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks.get(id).cloned()
    }

    pub async fn update(&self, id: &str, update: TaskUpdate) -> Option<Task> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(id) {
            if let Some(title) = update.title {
                task.title = title;
            }
            if let Some(description) = update.description {
                task.description = Some(description);
            }
            if let Some(status) = update.status {
                task.status = status;
            }
            if let Some(priority) = update.priority {
                task.priority = priority;
            }
            if let Some(tags) = update.tags {
                task.tags = tags;
            }
            task.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            return Some(task.clone());
        }
        None
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id).is_some()
    }

    pub async fn list(&self, options: TaskListOptions) -> Vec<Task> {
        let tasks = self.tasks.read().await;
        let mut result: Vec<Task> = tasks.values().cloned().collect();

        if let Some(status) = options.status {
            result.retain(|t| t.status == status);
        }

        if let Some(tags) = options.tags {
            result.retain(|t| tags.iter().any(|tag| t.tags.contains(tag)));
        }

        result.sort_by_key(|b| std::cmp::Reverse(b.created_at));

        if let Some(limit) = options.limit {
            result.truncate(limit);
        }

        result
    }
}
