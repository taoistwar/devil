use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronCreateInput {
    pub schedule: String,
    pub action: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronCreateOutput {
    pub id: String,
    pub schedule: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronDeleteInput {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronDeleteOutput {
    pub id: String,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronListOutput {
    pub jobs: Vec<CronJobInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobInfo {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub action: String,
}

pub struct CronStore {
    jobs: Arc<RwLock<HashMap<String, CronJobInfo>>>,
}

impl Default for CronStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CronStore {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create(&self, schedule: &str, action: &str, name: &str) -> CronJobInfo {
        let id = Uuid::new_v4().to_string();
        let job = CronJobInfo {
            id: id.clone(),
            name: name.to_string(),
            schedule: schedule.to_string(),
            action: action.to_string(),
        };
        let mut jobs = self.jobs.write().await;
        jobs.insert(id, job.clone());
        job
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut jobs = self.jobs.write().await;
        jobs.remove(id).is_some()
    }

    pub async fn list(&self) -> Vec<CronJobInfo> {
        let jobs = self.jobs.read().await;
        jobs.values().cloned().collect()
    }
}

pub struct CronCreateTool {
    store: CronStore,
}

impl CronCreateTool {
    pub fn new(store: CronStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for CronCreateTool {
    type Input = CronCreateInput;
    type Output = CronCreateOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "cron_create"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "schedule": {
                    "type": "string",
                    "description": "Cron schedule expression"
                },
                "action": {
                    "type": "string",
                    "description": "Action to perform"
                },
                "name": {
                    "type": "string",
                    "description": "Name of the cron job"
                }
            },
            "required": ["schedule", "action"]
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
        let name = input.name.unwrap_or_else(|| format!("job-{}", std::process::id()));
        let job = self.store.create(&input.schedule, &input.action, &name).await;

        let output = CronCreateOutput {
            id: job.id,
            schedule: job.schedule,
            success: true,
        };

        Ok(ToolResult::success("cron_create-1", output))
    }
}

pub struct CronDeleteTool {
    store: CronStore,
}

impl CronDeleteTool {
    pub fn new(store: CronStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for CronDeleteTool {
    type Input = CronDeleteInput;
    type Output = CronDeleteOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "cron_delete"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "ID of the cron job to delete"
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
        let success = self.store.delete(&input.id).await;

        let output = CronDeleteOutput {
            id: input.id,
            success,
        };

        Ok(ToolResult::success("cron_delete-1", output))
    }
}

pub struct CronListTool {
    store: CronStore,
}

impl CronListTool {
    pub fn new(store: CronStore) -> Self {
        Self { store }
    }
}

#[async_trait]
impl Tool for CronListTool {
    type Input = ();
    type Output = CronListOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "cron_list"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
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
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let jobs = self.store.list().await;

        let output = CronListOutput { jobs };

        Ok(ToolResult::success("cron_list-1", output))
    }
}
