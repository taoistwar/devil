use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::tools::tool::{
    Tool, ToolContext, ToolPermissionLevel, ToolProgress, ToolResult,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REPLInput {
    pub language: String,
    pub code: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct REPLOutput {
    pub session_id: String,
    pub result: String,
    pub error: Option<String>,
    pub output: Option<String>,
}

pub struct REPLTool {
    sessions: Arc<RwLock<HashMap<String, REPLSession>>>,
}

#[derive(Debug, Clone)]
pub struct REPLSession {
    pub language: String,
    pub context: HashMap<String, String>,
}

impl Default for REPLTool {
    fn default() -> Self {
        Self::new()
    }
}

impl REPLTool {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn get_or_create_session(
        &self,
        session_id: &str,
        language: &str,
    ) -> REPLSession {
        let mut sessions = self.sessions.write().await;
        sessions
            .entry(session_id.to_string())
            .or_insert_with(|| REPLSession {
                language: language.to_string(),
                context: HashMap::new(),
            })
            .clone()
    }
}

#[async_trait]
impl Tool for REPLTool {
    type Input = REPLInput;
    type Output = REPLOutput;
    type Progress = serde_json::Value;

    fn name(&self) -> &str {
        "repl"
    }

    fn aliases(&self) -> &[&str] {
        &["execute", "eval", "REPL"]
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "description": "Programming language (python, node, ruby, etc.)",
                    "enum": ["python", "node", "ruby", "bash", "lua", "perl"]
                },
                "code": {
                    "type": "string",
                    "description": "Code to execute"
                },
                "session_id": {
                    "type": "string",
                    "description": "Session ID for stateful REPL sessions"
                }
            },
            "required": ["language", "code"]
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

    async fn execute(
        &self,
        input: Self::Input,
        _ctx: &ToolContext,
        _progress_callback: Option<impl Fn(ToolProgress<Self::Progress>) + Send + Sync>,
    ) -> Result<ToolResult<Self::Output>> {
        let session_id = input.session_id.unwrap_or_else(|| {
            format!("repl-{}-{}", input.language, std::process::id())
        });

        let session = self.get_or_create_session(&session_id, &input.language).await;

        let (result, error, output) = execute_code(&input.language, &input.code, &session.context)?;

        let repl_output = REPLOutput {
            session_id: session_id.clone(),
            result,
            error,
            output,
        };

        Ok(ToolResult::success("repl-1", repl_output))
    }
}

fn execute_code(
    language: &str,
    code: &str,
    _context: &HashMap<String, String>,
) -> Result<(String, Option<String>, Option<String>)> {
    let output = match language.to_lowercase().as_str() {
        "python" => {
            let mut cmd = std::process::Command::new("python3");
            cmd.arg("-c").arg(code);
            cmd.output()?
        }
        "node" => {
            let mut cmd = std::process::Command::new("node");
            cmd.arg("-e").arg(code);
            cmd.output()?
        }
        "ruby" => {
            let mut cmd = std::process::Command::new("ruby");
            cmd.arg("-e").arg(code);
            cmd.output()?
        }
        "bash" | "sh" => {
            let mut cmd = std::process::Command::new("bash");
            cmd.arg("-c").arg(code);
            cmd.output()?
        }
        "lua" => {
            let mut cmd = std::process::Command::new("lua");
            cmd.arg("-e").arg(code);
            cmd.output()?
        }
        "perl" => {
            let mut cmd = std::process::Command::new("perl");
            cmd.arg("-e").arg(code);
            cmd.output()?
        }
        _ => {
            return Ok((
                String::new(),
                Some(format!("Unsupported language: {}", language)),
                None,
            ));
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    let exit_code = output.status.code().unwrap_or(-1);

    if exit_code == 0 {
        Ok((stdout, None, None))
    } else {
        Ok((String::new(), Some(stderr), Some(stdout)))
    }
}
