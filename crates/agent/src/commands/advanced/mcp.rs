//! /mcp 命令 - MCP 服务器管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct McpCommand;

impl McpCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for McpCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for McpCommand {
    fn name(&self) -> &str {
        "mcp"
    }

    fn description(&self) -> &str {
        "MCP 服务器管理"
    }

    fn usage(&self) -> &str {
        "/mcp [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::success_with_data(
                "MCP 服务器列表",
                serde_json::json!({
                    "servers": [],
                    "action": "list"
                }),
            );
        }

        CommandResult::success(format!("MCP 命令: {}", args.join(" ")))
    }
}
