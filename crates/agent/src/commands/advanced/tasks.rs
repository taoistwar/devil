//! /tasks 命令 - 任务管理

use crate::commands::cmd_trait::{CommandContext, CommandResult, SlashCommand};
use async_trait::async_trait;

pub struct TasksCommand;

impl TasksCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TasksCommand {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SlashCommand for TasksCommand {
    fn name(&self) -> &str {
        "tasks"
    }

    fn description(&self) -> &str {
        "任务管理"
    }

    fn usage(&self) -> &str {
        "/tasks [subcommand]"
    }

    async fn execute(&self, ctx: &CommandContext, args: &[&str]) -> CommandResult {
        if args.is_empty() {
            return CommandResult::success_with_data(
                "任务列表",
                serde_json::json!({
                    "action": "tasks",
                    "session_id": ctx.session_id,
                    "tasks": []
                }),
            );
        }

        CommandResult::success(format!("任务命令: {}", args.join(" ")))
    }
}
