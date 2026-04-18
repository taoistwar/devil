//! 钩子系统模块
//!
//! 钩子系统是 Agent 的"神经系统"，遵循观察者模式 + 责任链模式。
//! 权限管线决定 Agent"能否"执行操作，钩子系统决定执行"前后"发生什么。
//!
//! ## 钩子类型（6 种）
//!
//! - **Command**: Shell 命令执行，毫秒级，适合脚本检查
//! - **Prompt**: LLM 推理，秒级，适合内容审核
//! - **Agent**: LLM 多步，秒~分钟，适合测试验证
//! - **HTTP**: HTTP 请求，网络依赖，适合 CI 集成
//! - **Callback**: TS 回调，毫秒级，仅运行时存在
//! - **Function**: 运行时注册的函数 Hook
//!
//! ## 生命周期事件（26 种）
//!
//! 覆盖工具调用、用户交互、会话管理、子代理、压缩、权限等场景。
//!
//! ## 安全门禁（三层）
//!
//! 1. 全局禁用（disableAllHooks: true）
//! 2. 仅托管钩子（allowManagedHooksOnly: true）
//! 3. 工作区信任检查（防止供应链攻击）
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use agent::hooks::{HookRegistry, HookExecutor, HookEvent};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建注册表
//! let mut registry = HookRegistry::new();
//!
//! // 注册钩子
//! // ...
//!
//! // 执行钩子
//! let executor = HookExecutor::new();
//! let event = HookEvent::PreToolUse {
//!     tool_name: "Bash".to_string(),
//!     tool_input: std::collections::HashMap::new(),
//! };
//!
//! let hooks = registry.get_matching_hooks(&event);
//! for hook in hooks {
//!     let response = executor.execute(&hook.hook, &event).await?;
//!     if response.should_block() {
//!         println!("钩子阻止了执行：{:?}", response.stop_reason);
//!         break;
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub mod builtin;
pub mod events;
pub mod executor;
pub mod matcher;
pub mod registry;
pub mod response;
pub mod security;
pub mod types;

pub use events::HookEvent;
pub use executor::{HookError, HookExecutor};
pub use matcher::HookMatcher;
pub use registry::{
    HookConfigSnapshot, HookPriority, HookRegistry, HookSource, HookSourceType, RegisteredHook,
};
pub use response::{HookDecision, HookResponse, HookSpecificOutput, PermissionDecision};
pub use security::{HookSecurityConfig, HookSecurityGuard, SecurityCheckResult};
pub use types::{AgentHook, CommandHook, HookType, HttpHook, PromptHook, ShellType};
