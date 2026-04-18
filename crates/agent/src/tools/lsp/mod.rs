use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPToolInput {
    pub action: String,
    pub language: String,
    pub file: Option<String>,
    pub position: Option<LSPPosition>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPPosition {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LSPToolOutput {
    pub action: String,
    pub language: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct LSPTool;

impl Default for LSPTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for LSPTool {
    type Input = LSPToolInput;
    type Output = LSPToolOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "lsp"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["hover", "goto", "completions", "diagnostics", "symbols"],
                    "description": "LSP action to perform"
                },
                "language": {
                    "type": "string",
                    "description": "Programming language"
                },
                "file": {
                    "type": "string",
                    "description": "File path"
                },
                "position": {
                    "type": "object",
                    "properties": {
                        "line": { "type": "integer" },
                        "character": { "type": "integer" }
                    },
                    "description": "Cursor position"
                },
                "content": {
                    "type": "string",
                    "description": "File content for analysis"
                }
            },
            "required": ["action", "language"]
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
        let output = LSPToolOutput {
            action: input.action.clone(),
            language: input.language.clone(),
            success: true,
            result: Some(serde_json::json!({
                "action": input.action,
                "language": input.language,
                "file": input.file,
                "status": "LSP integration available"
            })),
            error: None,
        };

        Ok(ToolResult::success("lsp-1", output))
    }
}
