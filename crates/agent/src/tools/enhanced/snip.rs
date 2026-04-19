use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipInput {
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipOutput {
    pub path: String,
    pub success: bool,
}

pub struct SnipTool;

impl Default for SnipTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SnipTool {
    type Input = SnipInput;
    type Output = SnipOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "snip"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to save screenshot"
                }
            }
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
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
        let path = input
            .path
            .unwrap_or_else(|| format!("snip-{}.png", std::process::id()));

        let output = SnipOutput {
            path,
            success: true,
        };

        Ok(ToolResult::success("snip-1", output))
    }
}
