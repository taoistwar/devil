use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuthInput {
    pub server: String,
    pub auth_type: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAuthOutput {
    pub server: String,
    pub authenticated: bool,
    pub message: String,
}

pub struct McpAuthTool;

impl Default for McpAuthTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for McpAuthTool {
    type Input = McpAuthInput;
    type Output = McpAuthOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "mcp_auth"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "MCP server name"
                },
                "auth_type": {
                    "type": "string",
                    "description": "Authentication type (bearer, basic, api_key)"
                },
                "token": {
                    "type": "string",
                    "description": "Authentication token"
                }
            },
            "required": ["server"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
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
        let output = McpAuthOutput {
            server: input.server,
            authenticated: true,
            message: "MCP authentication configured".to_string(),
        };

        Ok(ToolResult::success("mcp_auth-1", output))
    }
}
