//! /vim 命令 - Vim 编辑模式

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct VimCommand;

impl VimCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for VimCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for VimCommand {
    fn name(&self) -> &str {
        "vim"
    }

    fn description(&self) -> &str {
        "Vim 编辑模式"
    }

    fn aliases(&self) -> &[&str] {
        &["v"]
    }

    fn usage(&self) -> &str {
        "/vim [file]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let file = args.first().map(|s| s.to_string()).unwrap_or_default();

        CommandResult::success_with_data(
            format!(
                "Vim 编辑模式: {}",
                if file.is_empty() {
                    "新文件".to_string()
                } else {
                    file.clone()
                }
            ),
            serde_json::json!({
                "action": "vim",
                "file": file,
                "session_id": ctx.session_id
            }),
        )
    }
}
