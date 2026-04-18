use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMultipleFilesInput {
    pub paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadMultipleFilesOutput {
    pub files: Vec<FileContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileContent {
    pub path: String,
    pub content: Option<String>,
    pub error: Option<String>,
}

pub struct ReadMultipleFilesTool;

impl Default for ReadMultipleFilesTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReadMultipleFilesTool {
    type Input = ReadMultipleFilesInput;
    type Output = ReadMultipleFilesOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "read_multiple_files"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "paths": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Paths of files to read"
                }
            },
            "required": ["paths"]
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
        let files: Vec<FileContent> = input
            .paths
            .into_iter()
            .map(|path| {
                let content = std::fs::read_to_string(&path).ok();
                FileContent {
                    path,
                    content,
                    error: None,
                }
            })
            .collect();

        let output = ReadMultipleFilesOutput { files };

        Ok(ToolResult::success("read_multiple_files-1", output))
    }
}
