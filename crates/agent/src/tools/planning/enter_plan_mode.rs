use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::planning::{AgentState, PlanMode};
use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::task::scheduler::TaskScheduler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterPlanModeInput {
    pub mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterPlanModeOutput {
    pub previous_state: String,
    pub current_state: String,
    pub mode: String,
}

pub struct EnterPlanModeTool {
    scheduler: TaskScheduler,
}

impl EnterPlanModeTool {
    pub fn new(scheduler: TaskScheduler) -> Self {
        Self { scheduler }
    }
}

fn parse_mode(s: &str) -> PlanMode {
    match s.to_lowercase().as_str() {
        "browse" => PlanMode::Browse,
        "task" => PlanMode::Task,
        "debug" => PlanMode::Debug,
        _ => PlanMode::default(),
    }
}

#[async_trait]
impl Tool for EnterPlanModeTool {
    type Input = EnterPlanModeInput;
    type Output = EnterPlanModeOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "enter_plan_mode"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["browse", "task", "debug"],
                    "description": "Planning mode to enter"
                }
            }
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::RequiresConfirmation
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let previous_state = self.scheduler.get_state().await;
        let previous_state_str = format!("{:?}", previous_state);

        let new_mode = input
            .mode
            .map(|s| parse_mode(&s))
            .unwrap_or(PlanMode::Task);

        let new_state = AgentState::Planning;
        self.scheduler.set_state(new_state).await;

        let output = EnterPlanModeOutput {
            previous_state: previous_state_str,
            current_state: format!("{:?}", new_state),
            mode: format!("{:?}", new_mode),
        };

        Ok(ToolResult::success("enter_plan_mode-1", output))
    }
}
