use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepInput {
    pub duration_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SleepOutput {
    pub duration_secs: u64,
    pub success: bool,
}

pub struct SleepTool;

impl Default for SleepTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SleepTool {
    type Input = SleepInput;
    type Output = SleepOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "sleep"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "duration_secs": {
                    "type": "integer",
                    "description": "Duration to sleep in seconds"
                }
            },
            "required": ["duration_secs"]
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
        tokio::time::sleep(Duration::from_secs(input.duration_secs)).await;

        let output = SleepOutput {
            duration_secs: input.duration_secs,
            success: true,
        };

        Ok(ToolResult::success("sleep-1", output))
    }
}
