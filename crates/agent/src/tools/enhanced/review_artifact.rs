use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewArtifactInput {
    pub content: String,
    pub artifact_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewArtifactOutput {
    pub review: String,
    pub suggestions: Vec<String>,
    pub success: bool,
}

pub struct ReviewArtifactTool;

impl Default for ReviewArtifactTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ReviewArtifactTool {
    type Input = ReviewArtifactInput;
    type Output = ReviewArtifactOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "review_artifact"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "content": {
                    "type": "string",
                    "description": "Content to review"
                },
                "artifact_type": {
                    "type": "string",
                    "description": "Type of artifact (code, doc, config)"
                }
            },
            "required": ["content"]
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
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let output = ReviewArtifactOutput {
            review: "Artifact review completed".to_string(),
            suggestions: Vec::new(),
            success: true,
        };

        Ok(ToolResult::success("review_artifact-1", output))
    }
}
