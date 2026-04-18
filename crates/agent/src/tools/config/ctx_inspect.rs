use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxInspectInput {
    pub depth: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtxInspectOutput {
    pub context_summary: String,
    pub working_directory: Option<String>,
    pub active_tools: Vec<String>,
    pub message_count: usize,
}

pub struct CtxInspectTool;

impl Default for CtxInspectTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CtxInspectTool {
    type Input = CtxInspectInput;
    type Output = CtxInspectOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "ctx_inspect"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "depth": {
                    "type": "integer",
                    "description": "Inspection depth (1-10)"
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
        input: Self::Input,
        ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let depth = input.depth.unwrap_or(5).min(10);

        let output = CtxInspectOutput {
            context_summary: format!("Context inspection at depth {}", depth),
            working_directory: ctx.working_directory.clone(),
            active_tools: Vec::new(),
            message_count: 0,
        };

        Ok(ToolResult::success("ctx_inspect-1", output))
    }
}
