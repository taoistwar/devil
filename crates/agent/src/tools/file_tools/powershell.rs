use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerShellInput {
    pub command: String,
    pub args: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerShellOutput {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub struct PowerShellTool;

impl Default for PowerShellTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for PowerShellTool {
    type Input = PowerShellInput;
    type Output = PowerShellOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "powershell"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "PowerShell command to execute"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Additional arguments"
                }
            },
            "required": ["command"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::Destructive
    }

    fn is_read_only(&self) -> bool {
        false
    }

    fn is_concurrency_safe(&self) -> bool {
        false
    }

    fn is_open_world(&self, _input: &Self::Input) -> bool {
        true
    }

    async fn execute(
        &self,
        _input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        #[cfg(target_os = "windows")]
        {
            let args = input.args.unwrap_or_default();
            let output = execute_powershell_internal(&input.command, &args)?;
            Ok(ToolResult::success("powershell-1", output))
        }

        #[cfg(not(target_os = "windows"))]
        {
            let output = PowerShellOutput {
                exit_code: 1,
                stdout: String::new(),
                stderr: "PowerShell is not available on this platform. This tool only works on Windows.".to_string(),
            };
            Ok(ToolResult {
                tool_use_id: "powershell-1".to_string(),
                is_success: false,
                output: Some(output),
                error: Some("PowerShell is not available on this platform".to_string()),
                context_modifier: None,
                interrupted: false,
            })
        }
    }
}

#[cfg(target_os = "windows")]
fn execute_powershell_internal(
    command: &str,
    _args: &[String],
) -> Result<PowerShellOutput> {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("pwsh");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", command]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    Ok(PowerShellOutput {
        exit_code: output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}
