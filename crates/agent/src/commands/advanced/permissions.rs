//! /permissions 命令 - 权限管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct PermissionsCommand;

impl PermissionsCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PermissionsCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for PermissionsCommand {
    fn name(&self) -> &str {
        "permissions"
    }

    fn description(&self) -> &str {
        "权限管理"
    }

    fn usage(&self) -> &str {
        "/permissions [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, _args: &[&str]) -> CommandResult {
        CommandResult::success_with_data(
            "权限管理",
            serde_json::json!({
                "action": "permissions",
                "session_id": ctx.session_id,
                "mode": "default"
            }),
        )
    }
}
