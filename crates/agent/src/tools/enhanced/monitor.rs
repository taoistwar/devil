use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInput {
    pub target: String,
    pub interval_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorOutput {
    pub target: String,
    pub metrics: SystemMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub uptime_secs: u64,
}

pub struct MonitorTool;

impl Default for MonitorTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MonitorTool {
    type Input = MonitorInput;
    type Output = MonitorOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "monitor"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "target": {
                    "type": "string",
                    "description": "Target to monitor (system, process, custom)"
                },
                "interval_secs": {
                    "type": "integer",
                    "description": "Monitoring interval"
                }
            },
            "required": ["target"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    fn is_read_only(&self) -> bool {
        true
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
        let output = MonitorOutput {
            target: input.target,
            metrics: SystemMetrics {
                cpu_percent: 0.0,
                memory_percent: 0.0,
                uptime_secs: 0,
            },
        };

        Ok(ToolResult::success("monitor-1", output))
    }
}
