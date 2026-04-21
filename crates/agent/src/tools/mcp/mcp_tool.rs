use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolInput {
    pub server: String,
    pub tool: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPToolOutput {
    pub server: String,
    pub tool: String,
    pub result: serde_json::Value,
    pub success: bool,
}

pub struct MCPTool;

impl Default for MCPTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MCPTool {
    type Input = MCPToolInput;
    type Output = MCPToolOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "mcp"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "MCP server name"
                },
                "tool": {
                    "type": "string",
                    "description": "Tool name on the MCP server"
                },
                "arguments": {
                    "type": "object",
                    "description": "Arguments to pass to the tool"
                }
            },
            "required": ["server", "tool"]
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

    fn timeout_ms(&self, _input: &Self::Input) -> Option<u64> {
        Some(60_000)
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = MCPToolOutput {
            server: input.server.clone(),
            tool: input.tool.clone(),
            result: serde_json::json!({
                "status": "MCP tool execution requires MCP server integration",
                "server": input.server,
                "tool": input.tool,
                "arguments": input.arguments
            }),
            success: true,
        };

        Ok(ToolResult::success("mcp-1", output))
    }
}
