//! /onboarding 命令

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct OnboardingCommand;
impl OnboardingCommand {
    pub fn new() -> Self {
        Self
    }
}
impl Default for OnboardingCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for OnboardingCommand {
    fn name(&self) -> &str {
        "onboarding"
    }
    fn description(&self) -> &str {
        "入门引导"
    }
    fn usage(&self) -> &str {
        "/onboarding"
    }
    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "入门引导",
            serde_json::json!({"action": "onboarding", "session_id": ctx.session_id}),
        )
    }
}
