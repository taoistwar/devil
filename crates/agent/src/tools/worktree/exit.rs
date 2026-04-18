use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::worktree::WorktreeManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitWorktreeInput {
    pub name: String,
    pub remove: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitWorktreeOutput {
    pub name: String,
    pub success: bool,
}

pub struct ExitWorktreeTool {
    manager: WorktreeManager,
}

impl ExitWorktreeTool {
    pub fn new(manager: WorktreeManager) -> Self {
        Self { manager }
    }

    async fn remove_git_worktree(&self, path: &std::path::Path) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "remove", path.to_str().unwrap_or(".")])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to remove git worktree: {}", stderr);
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for ExitWorktreeTool {
    type Input = ExitWorktreeInput;
    type Output = ExitWorktreeOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "exit_worktree"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the worktree to exit"
                },
                "remove": {
                    "type": "boolean",
                    "description": "Whether to remove the git worktree"
                }
            },
            "required": ["name"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
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
        let worktree = self.manager.get(&input.name).await;

        if let Some(state) = worktree {
            if input.remove.unwrap_or(false) {
                if let Err(e) = self.remove_git_worktree(&state.path).await {
                }
            }
        }

        let removed = self.manager.remove(&input.name).await;

        let output = ExitWorktreeOutput {
            name: input.name,
            success: removed,
        };

        Ok(ToolResult::success("exit_worktree-1", output))
    }
}
