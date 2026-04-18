//! 子代理类型定义
//!
//! 定义子代理的类型、配置和参数

use crate::message::Message;
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// 子代理类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentType {
    /// 通用目的子代理（全新上下文）
    GeneralPurpose,
    /// Fork 子代理（继承父级上下文）
    Fork,
    /// 自定义子代理类型
    Custom(String),
}

/// 子代理定义
pub struct SubagentDefinition {
    /// 子代理类型标识
    pub agent_type: &'static str,
    /// 使用时机说明
    pub when_to_use: &'static str,
    /// 可用工具列表（"*" 表示继承父级完整工具集）
    pub tools: &'static [&'static str],
    /// 最大回合数
    pub max_turns: u32,
    /// 模型配置（"inherit" 表示继承父级模型）
    pub model: ModelConfig,
    /// 权限模式
    pub permission_mode: PermissionMode,
    /// 来源（built-in / custom）
    pub source: SubagentSource,
    /// 系统提示生成器
    pub system_prompt_fn: Option<SystemPromptFn>,
}

/// 模型配置
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelConfig {
    /// 继承父级模型
    Inherit,
    /// 指定模型
    Specific(String),
    /// 按用途选择（sonnet/opus/haiku）
    ByPurpose(ModelPurpose),
}

/// 模型用途
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelPurpose {
    Sonnet,
    Opus,
    Haiku,
}

/// 权限模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// 独立权限检查
    Independent,
    /// 权限提示冒泡到父级终端
    Bubble,
    /// 继承父级权限规则
    Inherit,
}

/// 子代理来源
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentSource {
    /// 内置子代理
    Builtin,
    /// 自定义子代理
    Custom,
    /// 插件提供的子代理
    Plugin,
}

/// 系统提示生成器类型
pub type SystemPromptFn = Arc<dyn Fn() -> String + Send + Sync>;

impl Clone for SubagentDefinition {
    fn clone(&self) -> Self {
        Self {
            agent_type: self.agent_type.clone(),
            when_to_use: self.when_to_use.clone(),
            tools: self.tools.clone(),
            max_turns: self.max_turns,
            model: self.model.clone(),
            permission_mode: self.permission_mode.clone(),
            source: self.source.clone(),
            system_prompt_fn: self.system_prompt_fn.clone(),
        }
    }
}

impl std::fmt::Debug for SubagentDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubagentDefinition")
            .field("agent_type", &self.agent_type)
            .field("when_to_use", &self.when_to_use)
            .field("tools", &self.tools)
            .field("max_turns", &self.max_turns)
            .field("model", &self.model)
            .field("permission_mode", &self.permission_mode)
            .field("source", &self.source)
            .field("system_prompt_fn", &"<fn>")
            .finish()
    }
}

/// Fork 子代理配置
#[derive(Debug, Clone)]
pub struct ForkSubagentConfig {
    /// 是否启用 Fork 子代理
    pub enabled: bool,
    /// 占位符结果文本（所有 Fork 共享相同文本以最大化 Prompt Cache 命中）
    pub placeholder_result: String,
    /// Fork 指令标签
    pub boilerplate_tag: String,
    /// 指令前缀
    pub directive_prefix: String,
}

impl Default for ForkSubagentConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            placeholder_result: String::from("Fork started — processing in background"),
            boilerplate_tag: String::from("fork-boilerplate"),
            directive_prefix: String::from("DIRECTIVE: "),
        }
    }
}

/// 缓存安全参数（用于 Prompt Cache 共享）
#[derive(Debug, Clone)]
pub struct CacheSafeParams {
    /// 系统提示（必须与父级一致才能命中缓存）
    pub system_prompt: String,
    /// 用户上下文
    pub user_context: HashMap<String, String>,
    /// 系统上下文
    pub system_context: HashMap<String, String>,
    /// 工具上下文（包含工具集、模型等）
    pub tool_use_context: ToolUseContext,
    /// 父级消息（用于 Prompt Cache 共享）
    pub fork_context_messages: Vec<Message>,
}

/// 工具使用上下文
#[derive(Debug, Clone)]
pub struct ToolUseContext {
    /// 可用工具列表
    pub available_tools: Vec<String>,
    /// 渲染后的系统提示（字节级精确，用于缓存共享）
    pub rendered_system_prompt: String,
    /// 思考配置
    pub thinking_config: Option<ThinkingConfig>,
}

/// 思考配置
#[derive(Debug, Clone)]
pub struct ThinkingConfig {
    /// 是否启用思考
    pub enabled: bool,
    /// 思考预算（tokens）
    pub budget_tokens: Option<u32>,
}

/// 子代理启动参数
#[derive(Debug, Clone)]
pub struct SubagentParams {
    /// 启动消息
    pub prompt_messages: Vec<Message>,
    /// 缓存安全参数
    pub cache_safe_params: CacheSafeParams,
    /// 子代理类型
    pub subagent_type: SubagentType,
    /// 子代理指令
    pub directive: String,
    /// 最大回合数
    pub max_turns: Option<u32>,
    /// 输出 token 上限（注意：设置此值会改变预算 token，可能影响缓存）
    pub max_output_tokens: Option<u32>,
    /// 是否跳过边链转录（用于临时性 Fork）
    pub skip_transcript: bool,
    /// 是否跳过缓存写入（用于即用即弃的 Fork）
    pub skip_cache_write: bool,
    /// 是否在后台运行
    pub run_in_background: bool,
    /// 工作树隔离路径（如果有）
    pub worktree_path: Option<String>,
    /// 父级工作目录（用于路径转换）
    pub parent_cwd: Option<String>,
}

/// 子代理执行结果
#[derive(Debug, Clone)]
pub struct SubagentResult {
    /// 所有产生的消息
    pub messages: Vec<Message>,
    /// 累计 token 使用
    pub total_usage: Usage,
    /// 是否成功完成
    pub success: bool,
}

/// Token 使用统计
#[derive(Debug, Clone, Default)]
pub struct Usage {
    /// 输入 tokens
    pub input_tokens: u32,
    /// 输出 tokens
    pub output_tokens: u32,
    /// 思考 tokens
    pub thinking_tokens: u32,
    /// 缓存读取 tokens
    pub cache_read_tokens: u32,
    /// 缓存写入 tokens
    pub cache_write_tokens: u32,
}

/// 内置 Fork 子代理定义
pub const FORK_AGENT: SubagentDefinition = SubagentDefinition {
    agent_type: "fork",
    when_to_use: "隐式 Fork - 继承完整对话上下文。当省略 subagent_type 且 Fork 功能启用时触发。",
    tools: &["*"],
    max_turns: 200,
    model: ModelConfig::Inherit,
    permission_mode: PermissionMode::Bubble,
    source: SubagentSource::Builtin,
    system_prompt_fn: None,
};
