//! Agent crate - 提供核心 Agent 功能
//!
//! 基于 Claude Code 架构实现，包含：
//! - 异步生成器驱动的对话主循环
//! - 状态转换模型 (State + Continue + Terminal)
//! - 依赖注入模式提升测试可维护性
//! - 上下文预处理管线
//! - 工具系统管理

pub mod config;
pub mod context;
pub mod coordinator;
pub mod core;
pub mod deps;
pub mod hooks;
pub mod mcp_integration;
pub mod message;
pub mod permissions;
pub mod plugins;
pub mod prompts;
pub mod skills;
pub mod state;
pub mod subagent;
pub mod tools;

pub use config::AgentConfig;
pub use config::AgentConfigBuilder;
pub use core::Agent;
pub use core::AgentLoop;
pub use core::AgentStatus;
pub use deps::QueryDeps;
pub use message::AssistantMessage;
pub use message::Message as AgentMessage;
pub use message::SystemMessage;
pub use message::ToolUseSummaryMessage;
pub use message::UserMessage;
pub use permissions::PermissionAction;
pub use permissions::PermissionMode;
pub use permissions::PermissionRule;
pub use permissions::PermissionUpdate;
pub use permissions::RuleSource;
pub use permissions::ToolPermissionContext;
pub use state::Continue;
pub use state::ContinueReason;
pub use state::State;
pub use state::Terminal;
pub use state::TerminalReason;
pub use state::TransitionState;
pub use tools::Tool;
pub use tools::ToolContext;
pub use tools::ToolResult;
pub use tools::ToolUseBlock;
