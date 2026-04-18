use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticOutputInput {
    pub template: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntheticOutputOutput {
    pub result: String,
    pub success: bool,
}

pub struct SyntheticOutputTool;

impl Default for SyntheticOutputTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SyntheticOutputTool {
    type Input = SyntheticOutputInput;
    type Output = SyntheticOutputOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "synthetic_output"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "template": {
                    "type": "string",
                    "description": "Output template"
                },
                "data": {
                    "type": "object",
                    "description": "Data to populate template"
                }
            },
            "required": ["template"]
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
        let output = SyntheticOutputOutput {
            result: input.template,
            success: true,
        };

        Ok(ToolResult::success("synthetic_output-1", output))
    }
}
