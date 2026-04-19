//! /exit 命令 - 退出程序

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /exit 命令
pub struct ExitCommand;

impl ExitCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExitCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ExitCommand {
    fn name(&self) -> &str {
        "exit"
    }

    fn description(&self) -> &str {
        "退出程序"
    }

    fn aliases(&self) -> &[&str] {
        &["quit", "q"]
    }

    fn usage(&self) -> &str {
        "/exit"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Exit requested for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "再见!",
            serde_json::json!({
                "action": "exit",
                "session_id": ctx.session_id,
                "status": "exiting"
            }),
        )
    }
}
