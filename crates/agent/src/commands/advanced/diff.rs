//! /diff 命令 - 查看文件差异

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct DiffCommand;

impl DiffCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DiffCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for DiffCommand {
    fn name(&self) -> &str {
        "diff"
    }

    fn description(&self) -> &str {
        "查看文件差异"
    }

    fn usage(&self) -> &str {
        "/diff [file]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let file = args.first().map(|s| s.to_string()).unwrap_or_else(|| "[未指定文件]".to_string());

        CommandResult::success_with_data(
            format!("文件差异: {}", file),
            serde_json::json!({
                "action": "diff",
                "file": file,
                "session_id": ctx.session_id
            }),
        )
    }
}
