use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowToolInput {
    pub name: String,
    pub action: String,
    pub steps: Option<Vec<WorkflowStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowToolOutput {
    pub workflow: String,
    pub action: String,
    pub success: bool,
    pub results: Option<Vec<serde_json::Value>>,
}

pub struct WorkflowTool;

impl Default for WorkflowTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for WorkflowTool {
    type Input = WorkflowToolInput;
    type Output = WorkflowToolOutput;
    type Progress = String;

    fn name(&self) -> &str {
        "workflow"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Workflow name"
                },
                "action": {
                    "type": "string",
                    "enum": ["run", "create", "list", "stop"],
                    "description": "Workflow action"
                },
                "steps": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "tool": { "type": "string" },
                            "args": { "type": "object" }
                        }
                    },
                    "description": "Workflow steps"
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
        let output = WorkflowToolOutput {
            workflow: input.name,
            action: input.action,
            success: true,
            results: None,
        };

        Ok(ToolResult::success("workflow-1", output))
    }
}
