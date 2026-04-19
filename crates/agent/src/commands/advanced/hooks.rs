//! /hooks 命令 - Hook 管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct HooksCommand;

impl HooksCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for HooksCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for HooksCommand {
    fn name(&self) -> &str {
        "hooks"
    }

    fn description(&self) -> &str {
        "Hook 管理"
    }

    fn usage(&self) -> &str {
        "/hooks [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "Hook 管理",
            serde_json::json!({
                "action": "hooks",
                "session_id": ctx.session_id,
                "hooks": []
            }),
        )
    }
}
