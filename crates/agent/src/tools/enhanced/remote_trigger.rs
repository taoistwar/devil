use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTriggerInput {
    pub trigger_id: String,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteTriggerOutput {
    pub trigger_id: String,
    pub triggered: bool,
    pub result: Option<serde_json::Value>,
}

pub struct RemoteTriggerTool;

impl Default for RemoteTriggerTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for RemoteTriggerTool {
    type Input = RemoteTriggerInput;
    type Output = RemoteTriggerOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "remote_trigger"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "trigger_id": {
                    "type": "string",
                    "description": "Trigger identifier"
                },
                "payload": {
                    "type": "object",
                    "description": "Payload to send"
                }
            },
            "required": ["trigger_id"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = RemoteTriggerOutput {
            trigger_id: input.trigger_id,
            triggered: true,
            result: input.payload,
        };

        Ok(ToolResult::success("remote_trigger-1", output))
    }
}
