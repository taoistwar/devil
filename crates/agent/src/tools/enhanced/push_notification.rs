use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationInput {
    pub title: String,
    pub body: String,
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushNotificationOutput {
    pub success: bool,
    pub notification_id: String,
}

pub struct PushNotificationTool;

impl Default for PushNotificationTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for PushNotificationTool {
    type Input = PushNotificationInput;
    type Output = PushNotificationOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "push_notification"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "title": {
                    "type": "string",
                    "description": "Notification title"
                },
                "body": {
                    "type": "string",
                    "description": "Notification body"
                },
                "priority": {
                    "type": "string",
                    "enum": ["low", "normal", "high"],
                    "description": "Priority level"
                }
            },
            "required": ["title", "body"]
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
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = PushNotificationOutput {
            success: true,
            notification_id: uuid::Uuid::new_v4().to_string(),
        };

        Ok(ToolResult::success("push_notification-1", output))
    }
}
