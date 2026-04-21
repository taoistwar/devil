//! /coordinator 命令 - 协调器模式管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use crate::coordinator::{is_coordinator_mode, Orchestrator, CoordinatorConfig};
use async_trait::async_trait;

pub struct CoordinatorCommand;

impl CoordinatorCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CoordinatorCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for CoordinatorCommand {
    fn name(&self) -> &str {
        "coordinator"
    }

    fn description(&self) -> &str {
        "协调器模式管理 (status/on/off)"
    }

    fn usage(&self) -> &str {
        "/coordinator [status|on|off]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        let orchestrator = Orchestrator::new();
        let config = CoordinatorConfig::default();
        let enabled = is_coordinator_mode(&config);

        match args.first() {
            Some(&"status") | None => {
                let status = orchestrator
                    .get_coordinator_status(enabled, config.simple_mode)
                    .await;

                CommandResult::success_with_data(
                    "协调器状态",
                    serde_json::to_value(status).unwrap(),
                )
            }
            Some(&"on") => {
                CommandResult::success_with_data(
                    "协调器模式已启用",
                    serde_json::json!({
                        "action": "coordinator_enabled",
                        "session_id": ctx.session_id
                    }),
                )
            }
            Some(&"off") => {
                CommandResult::success_with_data(
                    "协调器模式已禁用",
                    serde_json::json!({
                        "action": "coordinator_disabled",
                        "session_id": ctx.session_id
                    }),
                )
            }
            _ => {
                CommandResult::success_with_data(
                    "协调器管理",
                    serde_json::json!({
                        "action": "help",
                        "usage": "/coordinator [status|on|off]",
                        "session_id": ctx.session_id
                    }),
                )
            }
        }
    }
}
