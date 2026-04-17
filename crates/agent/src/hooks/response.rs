//! 钩子响应协议
//! 
//! 定义钩子执行的 JSON 响应格式

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 钩子响应（标准 JSON Schema）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResponse {
    /// 是否继续执行（默认 true）
    #[serde(default = "default_true")]
    pub continue_flag: bool,
    
    /// 抑制输出（默认 false）
    #[serde(default)]
    pub suppress_output: bool,
    
    /// 停止原因（当 continue=false 时）
    #[serde(default)]
    pub stop_reason: Option<String>,
    
    /// 全局决策（approve/block）
    #[serde(default)]
    pub decision: Option<HookDecision>,
    
    /// 决策原因
    #[serde(default)]
    pub reason: Option<String>,
    
    /// 注入到上下文的系统消息
    #[serde(default)]
    pub system_message: Option<String>,
    
    /// 钩子特定输出（按事件类型不同而异）
    #[serde(default)]
    pub hook_specific_output: Option<HookSpecificOutput>,
    
    /// 标准输出（用于日志）
    #[serde(skip)]
    pub stdout: Option<String>,
    
    /// 标准错误
    #[serde(skip)]
    pub stderr: Option<String>,
    
    /// 退出码（用于 asyncRewake 模式）
    #[serde(skip)]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HookDecision {
    Approve,
    Block,
}

fn default_true() -> bool {
    true
}

/// 钩子特定输出（按事件类型组织）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookSpecificOutput {
    /// 钩子事件名称
    #[serde(default)]
    pub hook_event_name: Option<String>,
    
    /// ========== PreToolUse 专有字段 ==========
    /// 权限决策
    #[serde(default)]
    pub permission_decision: Option<PermissionDecision>,
    /// 权限决策原因
    #[serde(default)]
    pub permission_decision_reason: Option<String>,
    /// 修改后的工具输入
    #[serde(default)]
    pub updated_input: Option<HashMap<String, serde_json::Value>>,
    /// 额外上下文
    #[serde(default)]
    pub additional_context: Option<String>,
    
    /// ========== PostToolUse 专有字段 ==========
    /// 修改后的 MCP 工具输出
    #[serde(default)]
    pub updated_mcp_tool_output: Option<serde_json::Value>,
    
    /// ========== PermissionRequest 专有字段 ==========
    /// 权限决策（allow/deny）
    #[serde(default)]
    pub decision: Option<PermissionAction>,
    
    /// ========== PermissionDenied 专有字段 ==========
    /// 是否重试
    #[serde(default)]
    pub retry: bool,
    
    /// ========== Elicitation 专有字段 ==========
    /// 动作
    #[serde(default)]
    pub action: Option<String>,
    /// 内容
    #[serde(default)]
    pub content: Option<String>,
    
    /// ========== SessionStart/CwdChanged/FileChanged 专有字段 ==========
    /// 初始用户消息
    #[serde(default)]
    pub initial_user_message: Option<String>,
    /// 文件监控路径
    #[serde(default)]
    pub watch_paths: Option<Vec<String>>,
    
    /// ========== WorktreeCreate 专有字段 ==========
    /// Worktree 路径
    #[serde(default)]
    pub worktree_path: Option<String>,
}

/// 权限决策
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}

/// 权限操作
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PermissionAction {
    Allow,
    Deny,
}

/// 异步钩子响应（stdout 第一行检测）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncHookResponse {
    #[serde(default)]
    pub r#async: bool,
    /// 进程 ID
    #[serde(default)]
    pub process_id: Option<String>,
    /// 后台执行信息
    #[serde(default)]
    pub async_response: Option<AsyncHookInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncHookInfo {
    /// 异步任务 ID
    pub async_task_id: String,
    /// 预估时长（毫秒）
    pub estimated_duration_ms: Option<u64>,
    /// 状态消息
    pub status_message: Option<String>,
}

impl HookResponse {
    /// 创建继续执行的响应
    pub fn ok() -> Self {
        Self {
            continue_flag: true,
            suppress_output: false,
            stop_reason: None,
            decision: None,
            reason: None,
            system_message: None,
            hook_specific_output: None,
            stdout: None,
            stderr: None,
            exit_code: None,
        }
    }
    
    /// 创建阻止执行的响应
    pub fn block(reason: impl Into<String>) -> Self {
        Self {
            continue_flag: false,
            stop_reason: Some(reason.into()),
            decision: Some(HookDecision::Block),
            ..Self::ok()
        }
    }
    
    /// 添加额外上下文
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.hook_specific_output.get_or_insert_with(Default::default).additional_context = Some(context.into());
        self
    }
    
    /// 设置权限决策
    pub fn with_permission_decision(
        mut self,
        decision: PermissionDecision,
        reason: impl Into<String>,
    ) -> Self {
        self.hook_specific_output.get_or_insert_with(Default::default).permission_decision =
            Some(decision);
        self.hook_specific_output.get_or_insert_with(Default::default).permission_decision_reason =
            Some(reason.into());
        self
    }
    
    /// 设置修改后的输入
    pub fn with_updated_input(
        mut self,
        updated_input: HashMap<String, serde_json::Value>,
    ) -> Self {
        self.hook_specific_output.get_or_insert_with(Default::default).updated_input =
            Some(updated_input);
        self
    }
    
    /// 判断是否应该阻止执行
    pub fn should_block(&self) -> bool {
        !self.continue_flag || matches!(self.decision, Some(HookDecision::Block))
    }
}

impl Default for HookResponse {
    fn default() -> Self {
        Self::ok()
    }
}
