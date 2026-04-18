use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::planning::AgentState;
use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};
use crate::tools::task::scheduler::TaskScheduler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPlanModeInput;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitPlanModeOutput {
    pub previous_state: String,
    pub current_state: String,
}

pub struct ExitPlanModeTool {
    scheduler: TaskScheduler,
}

impl ExitPlanModeTool {
    pub fn new(scheduler: TaskScheduler) -> Self {
        Self { scheduler }
    }
}

#[async_trait]
impl Tool for ExitPlanModeTool {
    type Input = ExitPlanModeInput;
    type Output = ExitPlanModeOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "exit_plan_mode"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {}
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
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let previous_state = self.scheduler.get_state().await;
        let previous_state_str = format!("{:?}", previous_state);

        let new_state = AgentState::Idle;
        self.scheduler.set_state(new_state).await;

        let output = ExitPlanModeOutput {
            previous_state: previous_state_str,
            current_state: format!("{:?}", AgentState::Idle),
        };

        Ok(ToolResult::success("exit_plan_mode-1", output))
    }
}
