use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolInput {
    pub name: String,
    pub action: String,
    pub arguments: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillToolOutput {
    pub skill: String,
    pub action: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct SkillTool;

impl Default for SkillTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SkillTool {
    type Input = SkillToolInput;
    type Output = SkillToolOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "skill"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the skill to invoke"
                },
                "action": {
                    "type": "string",
                    "enum": ["execute", "load", "unload", "list"],
                    "description": "Action to perform"
                },
                "arguments": {
                    "type": "object",
                    "description": "Arguments to pass to the skill"
                }
            },
            "required": ["name", "action"]
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
        let output = SkillToolOutput {
            skill: input.name,
            action: input.action,
            success: true,
            result: input.arguments,
            error: None,
        };

        Ok(ToolResult::success("skill-1", output))
    }
}
