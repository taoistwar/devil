pub mod read_multiple;
pub mod write_diff;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestionInput {
    pub question: String,
    pub options: Option<Vec<QuestionOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskUserQuestionOutput {
    pub question: String,
    pub status: String,
}

pub struct AskUserQuestionTool;

impl Default for AskUserQuestionTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for AskUserQuestionTool {
    type Input = AskUserQuestionInput;
    type Output = AskUserQuestionOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "ask_user_question"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "question": {
                    "type": "string",
                    "description": "Question to ask the user"
                },
                "options": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "label": { "type": "string" },
                            "description": { "type": "string" }
                        }
                    },
                    "description": "Answer options"
                }
            },
            "required": ["question"]
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
        let output = AskUserQuestionOutput {
            question: input.question,
            status: "pending".to_string(),
        };

        Ok(ToolResult::success("ask_user_question-1", output))
    }
}
