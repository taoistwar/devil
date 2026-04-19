use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::worktree::WorktreeManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterWorktreeInput {
    pub name: String,
    pub path: Option<String>,
    pub create: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterWorktreeOutput {
    pub worktree: WorktreeInfo,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorktreeInfo {
    pub name: String,
    pub path: String,
    pub is_primary: bool,
}

pub struct EnterWorktreeTool {
    manager: WorktreeManager,
}

impl EnterWorktreeTool {
    pub fn new(manager: WorktreeManager) -> Self {
        Self { manager }
    }

    async fn create_git_worktree(&self, name: &str, path: &PathBuf) -> Result<()> {
        let output = Command::new("git")
            .args(["worktree", "add", path.to_str().unwrap_or("."), &format!("HEAD-{}", name)])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to create git worktree: {}", stderr);
        }

        Ok(())
    }
}

#[async_trait]
impl Tool for EnterWorktreeTool {
    type Input = EnterWorktreeInput;
    type Output = EnterWorktreeOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "enter_worktree"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "Name of the worktree"
                },
                "path": {
                    "type": "string",
                    "description": "Path where to create the worktree"
                },
                "create": {
                    "type": "boolean",
                    "description": "Whether to create a new git worktree"
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
        ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let worktree_path = if let Some(path) = input.path {
            PathBuf::from(path)
        } else {
            ctx.working_directory
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(format!(".worktrees/{}", input.name))
        };

        let create = input.create.unwrap_or(true);

        if create {
            if let Err(_e) = self.create_git_worktree(&input.name, &worktree_path).await {
            }
        }

        let is_primary = self.manager.list().await.is_empty();

        let state = self.manager.add(input.name.clone(), worktree_path.clone(), is_primary).await;

        let output = EnterWorktreeOutput {
            worktree: WorktreeInfo {
                name: state.name,
                path: state.path.to_string_lossy().to_string(),
                is_primary: state.is_primary,
            },
            success: true,
        };

        Ok(ToolResult::success("enter_worktree-1", output))
    }
}
