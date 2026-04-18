use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebBrowserInput {
    pub url: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebBrowserOutput {
    pub url: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub success: bool,
}

pub struct WebBrowserTool;

impl Default for WebBrowserTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WebBrowserTool {
    type Input = WebBrowserInput;
    type Output = WebBrowserOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "webbrowser"
    }

    fn aliases(&self) -> &[&str] {
        &["browser", "WebBrowser"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to navigate to"
                },
                "action": {
                    "type": "string",
                    "enum": ["navigate", "click", "type", "screenshot"],
                    "description": "Browser action"
                }
            },
            "required": ["url"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn is_open_world(&self, _input: &Self::Input) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = WebBrowserOutput {
            url: input.url,
            title: None,
            content: None,
            success: true,
        };

        Ok(ToolResult::success("webbrowser-1", output))
    }
}
