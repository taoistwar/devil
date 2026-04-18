//! 子代理系统模块
//!
//! 实现子智能体与 Fork 模式，支持上下文继承和 Prompt Cache 优化。
//!
//! ## 概述
//!
//! 子代理系统允许主 Agent  spawn 子代理执行任务，分为两种模式：
//!
//! ### Fork 子代理（上下文继承）
//!
//! - 继承父级完整对话上下文
//! - 共享 Prompt Cache 以最大化命中率
//! - 权限提示冒泡到父级终端
//! - 支持 git worktree 隔离
//!
//! ### 通用子代理（全新上下文）
//!
//! - 从零开始，不继承上下文
//! - 适用于独立任务
//!
//! ## 核心优势
//!
//! 1. **Prompt Cache 最大化**: 多个并行 fork 共享相同的 API 请求前缀
//! 2. **上下文完整性**: 子代理看到父级的所有历史消息、工具集和系统提示
//! 3. **权限冒泡**: 子代理的权限请求上浮到父级终端显示
//! 4. **Worktree 隔离**: 支持 git worktree 隔离，子代理在独立分支工作
//! 5. **递归防护**: 两层检查防止 fork 嵌套
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use agent::subagent::{SubagentExecutor, SubagentRegistry, SubagentParams, SubagentType};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建注册表
//! let registry = SubagentRegistry::new();
//!
//! // 创建执行器
//! let executor = SubagentExecutor::new()
//!     .with_fork_config(agent::subagent::types::ForkSubagentConfig {
//!         enabled: true,
//!         ..Default::default()
//!     });
//!
//! // 启动 Fork 子代理
//! let params = SubagentParams {
//!     subagent_type: SubagentType::Fork,
//!     directive: "研究这个模块的结构".to_string(),
//!     // ... 其他参数
//!     prompt_messages: vec![],
//!     cache_safe_params: todo!(),
//!     max_turns: None,
//!     max_output_tokens: None,
//!     skip_transcript: false,
//!     skip_cache_write: false,
//!     run_in_background: true,
//!     worktree_path: None,
//!     parent_cwd: None,
//! };
//!
//! let result = executor.execute(params).await?;
//! println!("子代理完成，产生 {} 条消息", result.messages.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## 架构设计
//!
//! ```
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    AgentTool.call()                          │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │              isForkSubagentEnabled()?                        │
//! │  • !coordinator_mode                                         │
//! │  • !non_interactive_session                                  │
//! │  • FORK_SUBAGENT feature flag                                │
//! └─────────────────────────────────────────────────────────────┘
//!                              │
//!              ┌───────────────┴───────────────┐
//!              │                               │
//!              ▼                               ▼
//!    ┌───────────────────┐           ┌───────────────────┐
//!    │  Fork 路径          │           │  普通路径          │
//!    │  (继承上下文)      │           │  (全新上下文)      │
//!    └───────────────────┘           └───────────────────┘
//!              │                               │
//!              ▼                               ▼
//!    ┌───────────────────┐           ┌───────────────────┐
//!    │  递归防护检查      │           │  解析 subagent_   │
//!    │  1. query_source   │           │  type            │
//!    │  2. 消息扫描        │           │                  │
//!    └───────────────────┘           └───────────────────┘
//!              │                               │
//!              ▼                               ▼
//!    ┌───────────────────┐           ┌───────────────────┐
//!    │  获取父级 system   │           │  获取子代理定义    │
//!    │  prompt (直传)     │           │  一般提示词       │
//!    └───────────────────┘           └───────────────────┘
//!              │                               │
//!              ▼                               ▼
//!    ┌───────────────────┐           ┌───────────────────┐
//!    │  buildForked      │           │  buildNormal      │
//!    │  Messages()       │           │  Messages()       │
//!    │  • 克隆父级 msg     │           │  • 新 user msg     │
//!    │  • 占位符 result   │           │  • 独立上下文      │
//!    │  • directive       │           │                  │
//!    └───────────────────┘           └───────────────────┘
//!              │                               │
//!              ▼                               ▼
//!    ┌─────────────────────────────────────────────────────┐
//!    │              runAgent()                               │
//!    │  • useExactTools: true (Fork)                         │
//!    │  • override.systemPrompt: 父级 (Fork)                 │
//!    │  • forkContextMessages: 父级消息 (Fork)               │
//!    └─────────────────────────────────────────────────────┘
//! ```
//!
//! ## Prompt Cache 优化
//!
//! | 优化点 | 实现 |
//! |--------|------|
//! | **相同 system prompt** | 直传 `renderedSystemPrompt`，避免重新渲染 |
//! | **相同工具集** | `useExactTools: true` 直接使用父级工具 |
//! | **相同 thinking config** | 继承父级思考配置 |
//! | **相同占位符结果** | 所有 fork 使用 `FORK_PLACEHOLDER_RESULT` 相同文本 |
//! | **ContentReplacementState 克隆** | 默认克隆父级替换状态 |
//!
//! ## 模块结构
//!
//! ```
//! subagent/
//! ├── mod.rs              # 模块入口
//! ├── types.rs            # 类型定义
//! ├── registry.rs         # 子代理注册表
//! ├── executor.rs         # 执行引擎
//! ├── context_inheritance.rs  # 上下文继承
//! ├── recursion_guard.rs      # 递归防护
//! ├── fork_subagent.rs        # Fork 核心逻辑（可选）
//! ├── cache_optimization.rs   # Prompt Cache 优化（可选）
//! └── worktree_isolation.rs   # Worktree 隔离（可选）
//! ```

pub mod context_inheritance;
pub mod executor;
pub mod recursion_guard;
pub mod registry;
pub mod types;

pub use context_inheritance::{
    build_inherited_messages, build_user_message_with_placeholder, create_cache_safe_params,
    extract_tool_use_ids, filter_incomplete_tool_calls, get_last_assistant_message,
};
pub use executor::{SubagentError, SubagentExecutor};
pub use recursion_guard::{
    build_child_message, build_worktree_notice, check_recursion_guard, is_fork_query_source,
    is_in_fork_child, RecursionGuardResult,
};
pub use registry::{parse_subagent_type, SubagentRegistry};
pub use types::{
    CacheSafeParams, ForkSubagentConfig, ModelConfig, ModelPurpose, PermissionMode,
    SubagentDefinition, SubagentParams, SubagentResult, SubagentSource, SubagentType,
    ThinkingConfig, ToolUseContext, Usage, FORK_AGENT,
};

/// 检查 Fork 子代理是否启用
///
/// 门控条件：
/// - 启用 FORK_SUBAGENT feature flag
/// - 不在 Coordinator 模式
/// - 不在非交互式会话（SDK/pipe 模式）
pub fn is_fork_subagent_enabled(
    feature_flag: bool,
    coordinator_mode: bool,
    non_interactive: bool,
) -> bool {
    if feature_flag {
        if coordinator_mode {
            return false;
        }
        if non_interactive {
            return false;
        }
        return true;
    }
    false
}
