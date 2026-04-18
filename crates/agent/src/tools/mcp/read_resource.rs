use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMcpResourceInput {
    pub uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMcpResourceOutput {
    pub uri: String,
    pub contents: Option<String>,
    pub mime_type: Option<String>,
    pub success: bool,
}

pub struct ReadMcpResourceTool;

impl Default for ReadMcpResourceTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadMcpResourceTool {
    type Input = ReadMcpResourceInput;
    type Output = ReadMcpResourceOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "read_mcp_resource"
    }

    fn aliases(&self) -> &[&str] {
        &["mcp_resource", "read_mcp"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "uri": {
                    "type": "string",
                    "description": "URI of the MCP resource to read"
                }
            },
            "required": ["uri"]
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
        let output = ReadMcpResourceOutput {
            uri: input.uri.clone(),
            contents: None,
            mime_type: None,
            success: false,
        };

        Ok(ToolResult::success("read_mcp_resource-1", output))
    }
}
