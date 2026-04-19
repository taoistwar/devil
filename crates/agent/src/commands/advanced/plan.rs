//! /plan 命令 - 计划模式

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PlanCommand;

impl PlanCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlanCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PlanCommand {
    fn name(&self) -> &str {
        "plan"
    }

    fn description(&self) -> &str {
        "进入计划模式"
    }

    fn usage(&self) -> &str {
        "/plan"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        tracing::info!("Entering plan mode for session: {}", ctx.session_id);

        CommandResult::success_with_data(
            "已进入计划模式",
            serde_json::json!({
                "action": "plan",
                "mode": "plan",
                "session_id": ctx.session_id
            }),
        )
    }
}
