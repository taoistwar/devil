use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestBackgroundPRInput {
    pub repo: String,
    pub branch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestBackgroundPROutput {
    pub repo: String,
    pub suggestion: String,
}

pub struct SuggestBackgroundPRTool;

impl Default for SuggestBackgroundPRTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SuggestBackgroundPRTool {
    type Input = SuggestBackgroundPRInput;
    type Output = SuggestBackgroundPROutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "suggest_background_pr"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "repo": {
                    "type": "string",
                    "description": "Repository"
                },
                "branch": {
                    "type": "string",
                    "description": "Branch name"
                }
            },
            "required": ["repo"]
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
        let output = SuggestBackgroundPROutput {
            repo: input.repo,
            suggestion: "Consider running PR checks in background".to_string(),
        };

        Ok(ToolResult::success("suggest_background_pr-1", output))
    }
}
