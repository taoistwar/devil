use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchInput {
    pub query: String,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSearchOutput {
    pub results: Vec<ToolMatch>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMatch {
    pub name: String,
    pub description: String,
    pub score: f64,
}

pub struct ToolSearchTool;

impl Default for ToolSearchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ToolSearchTool {
    type Input = ToolSearchInput;
    type Output = ToolSearchOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "tool_search"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "category": {
                    "type": "string",
                    "description": "Tool category filter"
                }
            },
            "required": ["query"]
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
        let output = ToolSearchOutput {
            results: Vec::new(),
            total: 0,
        };

        Ok(ToolResult::success("tool_search-1", output))
    }
}
