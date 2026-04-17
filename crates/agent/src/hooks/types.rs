//! 钩子类型定义
//! 
//! 定义 6 种钩子类型及其配置 Schema

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// 钩子类型枚举（6 种）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookType {
    /// Command 钩子 - Shell 命令执行
    Command(CommandHook),
    /// Prompt 钩子 - LLM 提示词评估
    Prompt(PromptHook),
    /// Agent 钩子 - 子代理执行验证
    Agent(AgentHook),
    /// HTTP 钩子 - HTTP 请求
    Http(HttpHook),
    /// Callback 钩子 - 内部回调函数（仅运行时）
    #[serde(skip)]
    Callback(CallbackHook),
    /// Function 钩子 - 运行时注册函数（仅运行时）
    #[serde(skip)]
    Function(FunctionHook),
}

/// Command 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHook {
    /// Shell 命令
    pub command: String,
    /// Shell 类型
    #[serde(default = "default_shell")]
    pub shell: ShellType,
    /// 条件过滤器（权限规则语法）
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// 超时时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 状态消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// 仅执行一次
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
    /// 异步执行
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub r#async: bool,
    /// 异步唤醒（退出码 2 时唤醒模型）
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub async_rewake: bool,
}

/// Shell 类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ShellType {
    Bash,
    PowerShell,
}

fn default_shell() -> ShellType {
    ShellType::Bash
}

/// Prompt 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptHook {
    /// LLM 提示词
    pub prompt: String,
    /// 条件过滤器
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// 超时时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 使用的模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 状态消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// 仅执行一次
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
}

/// Agent 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHook {
    /// 验证提示词
    pub prompt: String,
    /// 条件过滤器
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// 超时时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 使用的模型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// 状态消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// 仅执行一次
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
}

/// HTTP 钩子配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpHook {
    /// 请求 URL
    pub url: String,
    /// 条件过滤器
    #[serde(rename = "if", skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// 超时时间（秒）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// 请求头
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    /// 允许的环境变量
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_env_vars: Vec<String>,
    /// 状态消息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// 仅执行一次
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub once: bool,
}

/// Callback 钩子（仅运行时存在）
#[derive(Clone)]
pub struct CallbackHook {
    /// 回调函数
    pub callback: Box<dyn Fn(&HookEvent) -> HookResponse + Send + Sync>,
}

impl std::fmt::Debug for CallbackHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CallbackHook").field("callback", &"<fn>").finish()
    }
}

/// Function 钩子（仅运行时存在）
#[derive(Clone)]
pub struct FunctionHook {
    /// 函数名称
    pub name: String,
    /// 回调函数
    pub callback: Box<dyn Fn(&HookEvent) -> HookResponse + Send + Sync>,
}

impl std::fmt::Debug for FunctionHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunctionHook")
            .field("name", &self.name)
            .field("callback", &"<fn>")
            .finish()
    }
}
