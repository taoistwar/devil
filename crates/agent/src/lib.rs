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
pub mod message;
pub mod state;
pub mod tools;
pub mod core;
pub mod deps;
pub mod permissions;
pub mod hooks;
pub mod subagent;
pub mod coordinator;
pub mod skills;
pub mod plugins;

pub use config::AgentConfig;
pub use config::AgentConfigBuilder;
pub use message::Message as AgentMessage;
pub use message::UserMessage;
pub use message::AssistantMessage;
pub use message::SystemMessage;
pub use message::ToolUseSummaryMessage;
pub use state::State;
pub use state::Terminal;
pub use state::TerminalReason;
pub use state::Continue;
pub use state::ContinueReason;
pub use state::TransitionState;
pub use core::Agent;
pub use core::AgentStatus;
pub use core::AgentLoop;
pub use deps::QueryDeps;
pub use tools::Tool;
pub use tools::ToolContext;
pub use tools::ToolResult;
pub use tools::ToolUseBlock;
pub use permissions::PermissionMode;
pub use permissions::PermissionRule;
pub use permissions::PermissionAction;
pub use permissions::RuleSource;
pub use permissions::ToolPermissionContext;
pub use permissions::PermissionUpdate;
