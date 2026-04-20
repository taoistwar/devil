//! 协调器模式类型定义
//!
//! 定义协调器模式的核心类型和配置

use serde::{Deserialize, Serialize};

/// 协调器模式配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CoordinatorConfig {
    /// 是否启用协调器模式
    pub enabled: bool,
    /// Simple 模式（Worker 仅使用 Bash/Read/Edit）
    pub simple_mode: bool,
    /// Scratchpad 目录（跨 Worker 共享知识）
    pub scratchpad_dir: Option<String>,
    /// MCP 服务器列表
    pub mcp_servers: Vec<String>,
}

/// Worker Agent 定义
#[derive(Debug, Clone)]
pub struct WorkerAgent {
    /// Agent 类型标识
    pub agent_type: String,
    /// 使用时机说明
    pub when_to_use: String,
    /// 可用工具列表
    pub tools: Vec<String>,
    /// 系统提示
    pub system_prompt: String,
}

/// 内部编排工具（仅协调者可用，Worker 禁用）
pub const INTERNAL_ORCHESTRATION_TOOLS: &[&str] =
    &["TeamCreate", "TeamDelete", "SendMessage", "SyntheticOutput"];

/// 协调者可用工具（仅用于派发任务）
pub const COORDINATOR_TOOLS: &[&str] = &["Agent", "SendMessage", "TaskStop"];

/// Worker 默认可用工具（排除内部编排工具）
pub const DEFAULT_WORKER_TOOLS: &[&str] = &[
    "Bash",
    "Read",
    "Edit",
    "Write",
    "MultiEdit",
    "NotebookEdit",
    "Glob",
    "Grep",
    "LS",
    "TodoRead",
    "TodoWrite",
];

/// Simple 模式 Worker 工具（仅 Bash/Read/Edit）
pub const SIMPLE_WORKER_TOOLS: &[&str] = &["Bash", "Read", "Edit"];

/// 任务通知格式（Worker 结果返回给协调者）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotification {
    /// 任务 ID（Agent ID）
    pub task_id: String,
    /// 任务状态
    pub status: TaskStatus,
    /// 人类可读的状态摘要
    pub summary: String,
    /// Worker 的最终文本响应（可选）
    pub result: Option<String>,
    /// 使用情况统计（可选）
    pub usage: Option<TaskUsage>,
}

/// 任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// 已完成
    Completed,
    /// 失败
    Failed,
    /// 被停止
    Killed,
}

/// 任务使用情况统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUsage {
    /// 总 tokens
    pub total_tokens: u32,
    /// 工具调用次数
    pub tool_uses: u32,
    /// 执行时长（毫秒）
    pub duration_ms: u64,
}

/// 任务阶段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskPhase {
    /// 研究阶段
    Research,
    /// 综合阶段（协调者执行）
    Synthesis,
    /// 实现阶段
    Implementation,
    /// 验证阶段
    Verification,
}

impl TaskNotification {
    /// 创建已完成的任务通知
    pub fn completed(
        task_id: impl Into<String>,
        summary: impl Into<String>,
        result: Option<String>,
    ) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Completed,
            summary: summary.into(),
            result,
            usage: None,
        }
    }

    /// 创建失败的任务通知
    pub fn failed(task_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Failed,
            summary: format!("失败：{}", error.into()),
            result: None,
            usage: None,
        }
    }

    /// 创建被停止的任务通知
    pub fn killed(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            status: TaskStatus::Killed,
            summary: "任务已被停止".to_string(),
            result: None,
            usage: None,
        }
    }

    /// 设置使用情况统计
    pub fn with_usage(mut self, usage: TaskUsage) -> Self {
        self.usage = Some(usage);
        self
    }
}

/// Worker 任务指令
#[derive(Debug, Clone)]
pub struct WorkerDirective {
    /// 任务描述（3-5 个词）
    pub description: String,
    /// 任务提示词（必须自包含所有必要上下文）
    pub prompt: String,
    /// 任务目的说明（可选）
    pub purpose: Option<String>,
    /// 子代理类型（Worker 模式固定为 "worker"）
    pub subagent_type: String,
}

impl WorkerDirective {
    /// 创建研究任务指令
    pub fn research(description: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            prompt: prompt.into(),
            purpose: Some("此研究将为 PR 描述提供信息 — 重点关注面向用户的变更".to_string()),
            subagent_type: "worker".to_string(),
        }
    }

    /// 创建实现任务指令
    pub fn implement(description: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            prompt: prompt.into(),
            purpose: Some("实现指定的功能变更".to_string()),
            subagent_type: "worker".to_string(),
        }
    }

    /// 创建验证任务指令
    pub fn verify(description: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            prompt: prompt.into(),
            purpose: Some("验证代码变更是否正常工作".to_string()),
            subagent_type: "worker".to_string(),
        }
    }
}

/// 获取 Worker 可用工具列表
///
/// 根据配置返回不同的工具集
pub fn get_worker_tools(config: &CoordinatorConfig) -> Vec<String> {
    let tools = if config.simple_mode {
        SIMPLE_WORKER_TOOLS.to_vec()
    } else {
        DEFAULT_WORKER_TOOLS.to_vec()
    };

    // 过滤掉内部编排工具
    tools
        .into_iter()
        .filter(|name| !INTERNAL_ORCHESTRATION_TOOLS.contains(name))
        .map(|s| s.to_string())
        .collect()
}

/// 构建 Worker 用户上下文字符串
///
/// 用于告知协调者 Worker 可用的工具
pub fn build_worker_tools_context(config: &CoordinatorConfig) -> String {
    let tools = get_worker_tools(config);
    let tools_str = tools.join(", ");

    let mut content = format!(
        "Workers spawned via the Agent tool have access to these tools: {}",
        tools_str
    );

    // 添加 MCP 服务器信息
    if !config.mcp_servers.is_empty() {
        let servers = config.mcp_servers.join(", ");
        content.push_str(&format!(
            "\n\nWorkers also have access to MCP tools from connected MCP servers: {}",
            servers
        ));
    }

    // 添加 Scratchpad 信息
    if let Some(ref scratchpad_dir) = config.scratchpad_dir {
        content.push_str(&format!(
            "\n\nScratchpad directory: {}\nWorkers can read and write here without permission prompts. Use this for durable cross-worker knowledge.",
            scratchpad_dir
        ));
    }

    content
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoordinatorStatus {
    pub enabled: bool,
    pub active_workers: Vec<WorkerStatus>,
    pub simple_mode: bool,
    pub total_workers_spawned: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerStatus {
    pub task_id: String,
    pub description: String,
    pub phase: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_worker_tools_default() {
        let config = CoordinatorConfig::default();
        let tools = get_worker_tools(&config);

        assert!(tools.contains(&"Bash".to_string()));
        assert!(tools.contains(&"Read".to_string()));
        assert!(tools.contains(&"Edit".to_string()));
        assert!(!tools.contains(&"SendMessage".to_string()));
    }

    #[test]
    fn test_get_worker_tools_simple() {
        let config = CoordinatorConfig {
            simple_mode: true,
            ..Default::default()
        };
        let tools = get_worker_tools(&config);

        assert_eq!(tools.len(), 3);
        assert_eq!(tools, vec!["Bash", "Read", "Edit"]);
    }

    #[test]
    fn test_task_notification_completed() {
        let notification = TaskNotification::completed(
            "agent-123",
            "任务完成",
            Some("找到了 null pointer 问题".to_string()),
        );

        assert_eq!(notification.task_id, "agent-123");
        assert_eq!(notification.status, TaskStatus::Completed);
        assert_eq!(notification.summary, "任务完成");
        assert!(notification.result.is_some());
    }
}
