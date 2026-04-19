use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteDiffInput {
    pub path: String,
    pub old_string: String,
    pub new_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteDiffOutput {
    pub path: String,
    pub success: bool,
    pub diff: String,
}

pub struct WriteDiffTool;

impl Default for WriteDiffTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WriteDiffTool {
    type Input = WriteDiffInput;
    type Output = WriteDiffOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "write_diff"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file"
                },
                "old_string": {
                    "type": "string",
                    "description": "Original content"
                },
                "new_string": {
                    "type": "string",
                    "description": "Replacement content"
                }
            },
            "required": ["path", "old_string", "new_string"]
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

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let diff = format!(
            "--- a/{}\n+++ b/{}\n@@ -1 +1 @@\n-{}\n+{}",
            input.path, input.path, input.old_string, input.new_string
        );

        let output = WriteDiffOutput {
            path: input.path,
            success: true,
            diff,
        };

        Ok(ToolResult::success("write_diff-1", output))
    }
}
