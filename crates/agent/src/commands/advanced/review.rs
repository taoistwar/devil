//! /review 命令 - 代码审查

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct ReviewCommand;

impl ReviewCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReviewCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ReviewCommand {
    fn name(&self) -> &str {
        "review"
    }

    fn description(&self) -> &str {
        "代码审查"
    }

    fn usage(&self) -> &str {
        "/review [file]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let file = args
            .first()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "[未指定文件]".to_string());

        CommandResult::success_with_data(
            format!("代码审查: {}", file),
            serde_json::json!({
                "action": "review",
                "file": file,
                "session_id": ctx.session_id
            }),
        )
    }
}
