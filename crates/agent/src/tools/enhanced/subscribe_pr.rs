use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePRInput {
    pub repo: String,
    pub pr_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscribePROutput {
    pub repo: String,
    pub pr_number: u32,
    pub subscribed: bool,
}

pub struct SubscribePRTool;

impl Default for SubscribePRTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SubscribePRTool {
    type Input = SubscribePRInput;
    type Output = SubscribePROutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "subscribe_pr"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "repo": {
                    "type": "string",
                    "description": "Repository (owner/repo)"
                },
                "pr_number": {
                    "type": "integer",
                    "description": "PR number"
                }
            },
            "required": ["repo", "pr_number"]
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
        let output = SubscribePROutput {
            repo: input.repo,
            pr_number: input.pr_number,
            subscribed: true,
        };

        Ok(ToolResult::success("subscribe_pr-1", output))
    }
}
