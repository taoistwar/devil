use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMcpResourcesInput {
    pub server: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMcpResourcesOutput {
    pub server: Option<String>,
    pub resources: Vec<ResourceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub uri: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub mime_type: Option<String>,
}

pub struct ListMcpResourcesTool;

impl Default for ListMcpResourcesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListMcpResourcesTool {
    type Input = ListMcpResourcesInput;
    type Output = ListMcpResourcesOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "list_mcp_resources"
    }

    fn aliases(&self) -> &[&str] {
        &["list_mcp", "mcp_resources"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "server": {
                    "type": "string",
                    "description": "Filter by MCP server name"
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
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = ListMcpResourcesOutput {
            server: input.server,
            resources: Vec::new(),
        };

        Ok(ToolResult::success("list_mcp_resources-1", output))
    }
}
