//! 钩子事件定义
//!
//! 定义 26 个生命周期事件

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 钩子事件类型（26 种）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum HookEvent {
    // ========== 会话管理 ==========
    /// 会话启动
    SessionStart { source: String },
    /// 会话结束
    SessionEnd { reason: String },
    /// 初始化完成
    Setup { trigger: String },

    // ========== 用户交互 ==========
    /// 用户提交消息
    UserPromptSubmit { message: String },
    /// Agent 停止响应
    Stop,
    /// Agent 停止失败
    StopFailure { error: String },

    // ========== 工具执行 ==========
    /// 工具调用前
    PreToolUse {
        tool_name: String,
        tool_input: HashMap<String, serde_json::Value>,
    },
    /// 工具调用后（成功）
    PostToolUse {
        tool_name: String,
        tool_output: serde_json::Value,
    },
    /// 工具调用后（失败）
    PostToolUseFailure { tool_name: String, error: String },

    // ========== 权限管理 ==========
    /// 权限请求
    PermissionRequest {
        tool_name: String,
        permission_type: String,
    },
    /// 权限被拒
    PermissionDenied { tool_name: String, reason: String },

    // ========== 子代理 ==========
    /// 子代理启动
    SubagentStart { agent_type: String },
    /// 子代理停止
    SubagentStop { agent_type: String },

    // ========== 上下文压缩 ==========
    /// 压缩前
    PreCompact { trigger: String },
    /// 压缩后
    PostCompact { trigger: String },

    // ========== 协作 ==========
    /// Teammate 空闲
    TeammateIdle,
    /// 任务创建
    TaskCreated,
    /// 任务完成
    TaskCompleted,

    // ========== MCP ==========
    /// MCP 服务器请求用户输入
    Elicitation { mcp_server_name: String },
    /// Elicitation 结果返回
    ElicitationResult { mcp_server_name: String },

    // ========== 通知 ==========
    /// 系统通知事件
    Notification { notification_type: String },

    // ========== 环境 ==========
    /// 配置变更
    ConfigChange { source: String },
    /// 工作目录变更
    CwdChanged { new_path: String },
    /// 文件变更
    FileChanged { file_path: String },
    /// 指令加载
    InstructionsLoaded { load_reason: String },
    /// Worktree 创建
    WorktreeCreate { path: String },
    /// Worktree 移除
    WorktreeRemove { path: String },
}

impl HookEvent {
    /// 获取事件名称
    pub fn name(&self) -> &'static str {
        match self {
            HookEvent::SessionStart { .. } => "SessionStart",
            HookEvent::SessionEnd { .. } => "SessionEnd",
            HookEvent::Setup { .. } => "Setup",
            HookEvent::UserPromptSubmit { .. } => "UserPromptSubmit",
            HookEvent::Stop => "Stop",
            HookEvent::StopFailure { .. } => "StopFailure",
            HookEvent::PreToolUse { .. } => "PreToolUse",
            HookEvent::PostToolUse { .. } => "PostToolUse",
            HookEvent::PostToolUseFailure { .. } => "PostToolUseFailure",
            HookEvent::PermissionRequest { .. } => "PermissionRequest",
            HookEvent::PermissionDenied { .. } => "PermissionDenied",
            HookEvent::SubagentStart { .. } => "SubagentStart",
            HookEvent::SubagentStop { .. } => "SubagentStop",
            HookEvent::PreCompact { .. } => "PreCompact",
            HookEvent::PostCompact { .. } => "PostCompact",
            HookEvent::TeammateIdle => "TeammateIdle",
            HookEvent::TaskCreated => "TaskCreated",
            HookEvent::TaskCompleted => "TaskCompleted",
            HookEvent::Elicitation { .. } => "Elicitation",
            HookEvent::ElicitationResult { .. } => "ElicitationResult",
            HookEvent::Notification { .. } => "Notification",
            HookEvent::ConfigChange { .. } => "ConfigChange",
            HookEvent::CwdChanged { .. } => "CwdChanged",
            HookEvent::FileChanged { .. } => "FileChanged",
            HookEvent::InstructionsLoaded { .. } => "InstructionsLoaded",
            HookEvent::WorktreeCreate { .. } => "WorktreeCreate",
            HookEvent::WorktreeRemove { .. } => "WorktreeRemove",
        }
    }

    /// 获取匹配字段（用于钩子过滤）
    pub fn match_field(&self) -> Option<String> {
        match self {
            HookEvent::PreToolUse { tool_name, .. } => Some(tool_name.clone()),
            HookEvent::PostToolUse { tool_name, .. } => Some(tool_name.clone()),
            HookEvent::PostToolUseFailure { tool_name, .. } => Some(tool_name.clone()),
            HookEvent::PermissionRequest { tool_name, .. } => Some(tool_name.clone()),
            HookEvent::PermissionDenied { tool_name, .. } => Some(tool_name.clone()),
            HookEvent::SubagentStart { agent_type, .. } => Some(agent_type.clone()),
            HookEvent::SubagentStop { agent_type, .. } => Some(agent_type.clone()),
            HookEvent::Notification {
                notification_type, ..
            } => Some(notification_type.clone()),
            HookEvent::Elicitation {
                mcp_server_name, ..
            } => Some(mcp_server_name.clone()),
            HookEvent::ElicitationResult {
                mcp_server_name, ..
            } => Some(mcp_server_name.clone()),
            _ => None,
        }
    }
}
