use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCaptureInput {
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalCaptureOutput {
    pub session_id: String,
    pub output: String,
    pub success: bool,
}

pub struct TerminalCaptureTool;

impl Default for TerminalCaptureTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TerminalCaptureTool {
    type Input = TerminalCaptureInput;
    type Output = TerminalCaptureOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "terminal_capture"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": {
                    "type": "string",
                    "description": "Terminal session ID"
                }
            }
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
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
        let session_id = input.session_id.unwrap_or_else(|| "default".to_string());

        let output = TerminalCaptureOutput {
            session_id,
            output: String::new(),
            success: true,
        };

        Ok(ToolResult::success("terminal_capture-1", output))
    }
}
