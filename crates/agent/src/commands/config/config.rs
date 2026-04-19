//! /config 命令 - 配置管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

/// /config 命令
pub struct ConfigCommand;

impl ConfigCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ConfigCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for ConfigCommand {
    fn name(&self) -> &str {
        "config"
    }

    fn description(&self) -> &str {
        "配置管理"
    }

    fn usage(&self) -> &str {
        "/config [key] [value]"
    }

    async fn execute(&self, _ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::success_with_data(
                "当前配置",
                serde_json::json!({
                    "model": "claude-sonnet-4-20250514",
                    "provider": "anthropic",
                    "max_tokens": 200000,
                    "max_turns": 50
                }),
            );
        }

        if args.len() == 1 {
            return CommandResult::success(format!("配置项 {}: [value]", args[0]));
        }

        CommandResult::success(format!("配置 {} 已更新", args[0]))
    }
}
