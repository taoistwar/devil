use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefInput {
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BriefOutput {
    pub summary: String,
    pub relevant_tools: Vec<String>,
}

pub struct BriefTool;

impl Default for BriefTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for BriefTool {
    type Input = BriefInput;
    type Output = BriefOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "brief"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "What information do you need?"
                }
            }
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
        let output = BriefOutput {
            summary: "Brief tool provides concise summaries of available tools and capabilities".to_string(),
            relevant_tools: vec![
                "bash".to_string(),
                "read".to_string(),
                "edit".to_string(),
                "write".to_string(),
                "glob".to_string(),
                "grep".to_string(),
            ],
        };

        Ok(ToolResult::success("brief-1", output))
    }
}
