//! 钩子执行引擎
//! 
//! 实现各类钩子的执行逻辑

use crate::hooks::types::{HookType, CommandHook, PromptHook, AgentHook, HttpHook, ShellType};
use crate::hooks::events::HookEvent;
use crate::hooks::response::HookResponse;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

/// 钩子执行器
pub struct HookExecutor {
    /// 默认超时时间（秒）
    default_timeout: u64,
    /// 环境变量
    envs: HashMap<String, String>,
}

impl HookExecutor {
    /// 创建执行器
    pub fn new() -> Self {
        Self {
            default_timeout: 600,
            envs: HashMap::new(),
        }
    }
    
    /// 设置环境变量
    pub fn with_envs(mut self, envs: HashMap<String, String>) -> Self {
        self.envs = envs;
        self
    }
    
    /// 执行钩子
    pub async fn execute(&self, hook: &HookType, event: &HookEvent) -> Result<HookResponse, HookError> {
        match hook {
            HookType::Command(cmd) => self.execute_command(cmd, event).await,
            HookType::Prompt(prompt) => self.execute_prompt(prompt, event).await,
            HookType::Agent(agent) => self.execute_agent(agent, event).await,
            HookType::Http(http) => self.execute_http(http, event).await,
            HookType::Callback(callback) => Ok((callback.callback)(event)),
            HookType::Function(func) => Ok((func.callback)(event)),
        }
    }
    
    /// 执行 Command 钩子
    async fn execute_command(&self, hook: &CommandHook, event: &HookEvent) -> Result<HookResponse, HookError> {
        let timeout_secs = hook.timeout.unwrap_or(self.default_timeout);
        let (shell, args) = self.build_shell_command(hook);
        
        let mut cmd = Command::new(shell);
        cmd.args(&args);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        for (key, value) in &self.envs {
            cmd.env(key, value);
        }
        if let Ok(cwd) = std::env::current_dir() {
            cmd.env("CLAUDE_PROJECT_DIR", cwd.to_string_lossy().as_ref());
        }
        
        let mut child = cmd.spawn()?;
        
        if let Some(mut stdin) = child.stdin.take() {
            let json_input = serde_json::to_string(event)?;
            stdin.write_all(json_input.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
        }
        
        let stdout_reader = BufReader::new(child.stdout.take().ok_or(HookError::NoStdout)?);
        let stderr_reader = BufReader::new(child.stderr.take().ok_or(HookError::NoStderr)?);
        
        let mut stdout_lines = stdout_reader.lines();
        let mut stderr_lines = stderr_reader.lines();
        
        let mut stdout_content = String::new();
        let mut stderr_content = String::new();
        let mut exit_code: Option<i32> = None;
        
        let result = timeout(Duration::from_secs(timeout_secs), async {
            if let Ok(Some(line)) = stdout_lines.next_line().await {
                if line.trim().starts_with("{\"async\":true}") {
                    return self.handle_async_hook(&line).await;
                }
                stdout_content.push_str(&line);
                stdout_content.push('\n');
            }
            
            while let Ok(Some(line)) = stdout_lines.next_line().await {
                stdout_content.push_str(&line);
                stdout_content.push('\n');
            }
            
            while let Ok(Some(line)) = stderr_lines.next_line().await {
                stderr_content.push_str(&line);
                stderr_content.push('\n');
            }
            
            let status = child.wait().await?;
            exit_code = status.code();
            
            Ok::<HookResponse, HookError>(HookResponse::ok())
        }).await;
        
        match result {
            Ok(Ok(_)) => {
                let response = self.parse_command_response(&stdout_content, exit_code, stderr_content)?;
                Ok(response)
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(HookError::Timeout(timeout_secs)),
        }
    }
    
    fn build_shell_command(&self, hook: &CommandHook) -> (&'static str, Vec<String>) {
        match hook.shell {
            ShellType::Bash => ("bash", vec!["-c".to_string(), hook.command.clone()]),
            ShellType::PowerShell => ("pwsh", vec![
                "-NoProfile".to_string(),
                "-NonInteractive".to_string(),
                "-Command".to_string(),
                hook.command.clone(),
            ]),
        }
    }
    
    fn parse_command_response(
        &self,
        stdout: &str,
        exit_code: Option<i32>,
        stderr: String,
    ) -> Result<HookResponse, HookError> {
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') {
                if let Ok(mut response) = serde_json::from_str::<HookResponse>(trimmed) {
                    response.stdout = Some(stdout.to_string());
                    response.stderr = Some(stderr);
                    response.exit_code = exit_code;
                    return Ok(response);
                }
            }
        }
        
        let response = match exit_code {
            Some(0) => HookResponse::ok(),
            Some(1) => HookResponse::block("钩子执行失败"),
            Some(2) => HookResponse::ok().with_context("异步钩子唤醒请求"),
            _ => HookResponse::block(format!("未知退出码：{:?}", exit_code)),
        };
        
        Ok(HookResponse {
            stdout: Some(stdout.to_string()),
            stderr: Some(stderr),
            exit_code,
            ..response
        })
    }
    
    async fn handle_async_hook(&self, json_line: &str) -> Result<HookResponse, HookError> {
        if let Ok(async_response) = serde_json::from_str::<crate::hooks::response::AsyncHookResponse>(json_line) {
            let mut response = HookResponse::ok();
            response.continue_flag = true;
            Ok(response)
        } else {
            Ok(HookResponse::ok())
        }
    }
    
    async fn execute_prompt(&self, hook: &PromptHook, event: &HookEvent) -> Result<HookResponse, HookError> {
        let mut prompt = hook.prompt.clone();
        if let Some(pos) = prompt.find("$ARGUMENTS") {
            let event_json = serde_json::to_string(event)?;
            prompt.replace_range(pos..pos + 12, &event_json);
        }
        
        let mut response = HookResponse::ok();
        response.system_message = Some(format!("Prompt 钩子：{}", prompt));
        Ok(response)
    }
    
    async fn execute_agent(&self, hook: &AgentHook, event: &HookEvent) -> Result<HookResponse, HookError> {
        let mut prompt = hook.prompt.clone();
        if let Some(pos) = prompt.find("$ARGUMENTS") {
            let event_json = serde_json::to_string(event)?;
            prompt.replace_range(pos..pos + 12, &event_json);
        }
        
        let mut response = HookResponse::ok();
        response.system_message = Some(format!("Agent 钩子验证：{}", prompt));
        Ok(response)
    }
    
    async fn execute_http(&self, hook: &HttpHook, event: &HookEvent) -> Result<HookResponse, HookError> {
        let client = reqwest::Client::new();
        let event_json = serde_json::to_string(event)?;
        
        let mut request = client.post(&hook.url)
            .header("Content-Type", "application/json");
        
        for (key, value) in &hook.headers {
            request = request.header(key, value);
        }
        
        let response = request.body(event_json).send().await?;
        let status = response.status();
        
        if status.is_success() {
            Ok(HookResponse::ok())
        } else {
            Err(HookError::HttpError(format!("HTTP 请求失败：{}", status)))
        }
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// 钩子错误
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("钩子执行超时：{0}秒")]
    Timeout(u64),
    
    #[error("没有标准输出")]
    NoStdout,
    
    #[error("没有标准错误")]
    NoStderr,
    
    #[error("JSON 序列化失败：{0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO 错误：{0}")]
    IoError(#[from] std::io::Error),
    
    #[error("HTTP 错误：{0}")]
    HttpError(String),
    
    #[error("系统错误：{0}")]
    SystemError(String),
}

impl From<std::path::PathBuf> for HookError {
    fn from(err: std::path::PathBuf) -> Self {
        HookError::SystemError(format!("路径错误：{:?}", err))
    }
}

impl From<reqwest::Error> for HookError {
    fn from(err: reqwest::Error) -> Self {
        HookError::HttpError(format!("请求失败：{}", err))
    }
}
