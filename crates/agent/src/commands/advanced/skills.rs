//! /skills 命令 - 技能管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct SkillsCommand;

impl SkillsCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SkillsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for SkillsCommand {
    fn name(&self) -> &str {
        "skills"
    }

    fn description(&self) -> &str {
        "技能管理"
    }

    fn usage(&self) -> &str {
        "/skills [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::success_with_data(
                "技能列表",
                serde_json::json!({
                    "action": "skills",
                    "session_id": ctx.session_id,
                    "skills": []
                }),
            );
        }

        CommandResult::success(format!("技能命令: {}", args.join(" ")))
    }
}
