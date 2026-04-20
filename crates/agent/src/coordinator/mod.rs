//! 协调器模式模块
//!
//! 实现多 Agent 编排功能，将 CLI 变为"编排者"角色。
//!
//! ## 概述
//!
//! 协调器模式将 Agent 变为编排者角色。编排者不直接操作文件，而是通过 Agent 工具派发任务给多个 worker 并行执行。
//! 适用于大型任务拆分、并行研究、实现 + 验证分离等场景。
//!
//! ## 核心约束
//!
//! - 编排者只能使用：`Agent`（派发 worker）、`SendMessage`（继续 worker）、`TaskStop`（停止 worker）
//! - Worker 可以使用所有标准工具（Bash、Read、Edit 等）+ MCP 工具 + Skill 工具
//! - 编排者的每条消息都是给用户看的；worker 结果以 `<task-notification>` XML 形式到达
//!
//! ## 启用方式
//!
//! ```bash
//! # 基本启用
//! FEATURE_COORDINATOR_MODE=1 CLAUDE_CODE_COORDINATOR_MODE=1 bun run dev
//!
//! # 配合 Fork Subagent
//! FEATURE_COORDINATOR_MODE=1 FEATURE_FORK_SUBAGENT=1 \
//! CLAUDE_CODE_COORDINATOR_MODE=1 bun run dev
//!
//! # Simple 模式（worker 只有 Bash/Read/Edit）
//! FEATURE_COORDINATOR_MODE=1 CLAUDE_CODE_COORDINATOR_MODE=1 \
//! CLAUDE_CODE_SIMPLE=1 bun run dev
//! ```
//!
//! ## 典型工作流
//!
//! ```
//! 用户："修复 auth 模块的 null pointer"
//!
//! 编排者:
//!   1. 并行派发两个 worker:
//!      - Agent({ description: "调查 auth bug", prompt: "..." })
//!      - Agent({ description: "研究 auth 测试", prompt: "..." })
//!
//!   2. 收到 <task-notification>:
//!      - Worker A: "在 validate.ts:42 发现 null pointer"
//!      - Worker B: "测试覆盖情况..."
//!
//!   3. 综合发现，继续 Worker A:
//!      - SendMessage({ to: "agent-a1b", message: "修复 validate.ts:42..." })
//!
//!   4. 收到修复结果，派发验证:
//!      - Agent({ description: "验证修复", prompt: "..." })
//! ```
//!
//! ## 任务阶段
//!
//! | 阶段 | 执行者 | 目的 |
//! |------|-------|------|
//! | Research | Workers (并行) | 调查代码库，查找文件，理解问题 |
//! | Synthesis | **协调者** | 阅读发现，理解问题，编写实现规范 |
//! | Implementation | Workers | 根据规范进行有针对性的变更，提交 |
//! | Verification | Workers | 测试变更是否有效 |
//!
//! ## 核心设计决策
//!
//! 1. **双开关设计**：feature flag 控制代码可用性，环境变量控制实际激活
//! 2. **编排者受限**：只能用 Agent/SendMessage/TaskStop，确保编排者专注于派发而非执行
//! 3. **Worker 不可见编排者对话**：每个 worker 的 prompt 必须自包含（所有必要上下文）
//! 4. **并行优先**：系统提示强调"Parallelism is your superpower"，鼓励并行派发独立任务
//! 5. **综合而非转发**：编排者必须理解 worker 发现，再写出具体的实现指令
//! 6. **Scratchpad 可选共享**：通过 GrowthBook 门控的共享目录，让 worker 之间持久化共享知识
//!
//! ## 继续 vs. 派发决策
//!
//! | 情况 | 机制 | 原因 |
//! |------|------|------|
//! | 研究正好探索了需要编辑的文件 | **Continue** (SendMessage) + 综合规范 | Worker 已经有文件在上下文中，现在获得清晰的计划 |
//! | 研究很广泛但实现很窄 | **Spawn fresh** (Agent) + 综合规范 | 避免带着探索噪音；聚焦上下文更清晰 |
//! | 纠正失败或扩展最近工作 | **Continue** | Worker 有错误上下文，知道刚才尝试了什么 |
//! | 验证另一个 worker 刚写的代码 | **Spawn fresh** | 验证者应该以新鲜视角看代码，不携带实现假设 |
//! | 第一次实现尝试完全用错了方法 | **Spawn fresh** | 错误方法的上下文污染重试；干净 slate 避免锚定失败路径 |
//! | 完全无关的任务 | **Spawn fresh** | 没有有用的上下文可重用 |
//!
//! ## 模块结构
//!
//! ```
//! coordinator/
//! ├── mod.rs              # 模块入口
//! ├── types.rs            # 类型定义
//! ├── mode_detection.rs   # 模式检测
//! ├── system_prompt.rs    # 系统提示生成
//! ├── worker_agent.rs     # Worker Agent 定义
//! └── orchestration.rs    # 任务编排逻辑
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use agent::coordinator::{Orchestrator, WorkerDirective, CoordinatorConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建配置
//! let config = CoordinatorConfig {
//!     enabled: true,
//!     simple_mode: false,
//!     scratchpad_dir: Some("/tmp/scratchpad".to_string()),
//!     mcp_servers: vec![],
//! };
//!
//! // 创建编排器
//! let orchestrator = Orchestrator::new();
//!
//! // 并行派发研究任务
//! let task1 = orchestrator.spawn_research(
//!     "调查 auth bug",
//!     "研究 src/auth/validate.ts 中的空指针问题",
//! ).await;
//!
//! let task2 = orchestrator.spawn_research(
//!     "研究 token 存储",
//!     "研究安全的 token 存储方案",
//! ).await;
//!
//! println!("派发了两个任务：{} 和 {}", task1, task2);
//!
//! // 等待任务完成...
//!
//! // 综合研究结果
//! // orchestrator.synthesize_research(findings).await;
//!
//! // 派发实现任务
//! orchestrator.spawn_implementation(
//!     "修复 auth bug",
//!     "在 src/auth/validate.ts:42 添加空值检查...",
//! ).await;
//! # Ok(())
//! # }
//! ```
//!
//! ## 与 Claude Code 对齐
//!
//! | Claude Code 文件 | 本实现 |
//! |-----------------|--------|
//! | `src/coordinator/coordinatorMode.ts` | `mode_detection.rs`, `system_prompt.rs` |
//! | `src/coordinator/workerAgent.ts` | `worker_agent.rs` |
//! | `docs/features/coordinator-mode.md` | 本文档 + `types.rs` |

pub mod mode_detection;
pub mod orchestration;
pub mod system_prompt;
pub mod types;
pub mod worker_agent;

pub use mode_detection::{
    disable_coordinator_mode, enable_coordinator_mode, get_coordinator_user_context,
    is_coordinator_mode, match_session_mode, SessionMode,
};
pub use orchestration::{ContinueOrSpawn, Orchestrator, PromptBuilder, RunningTask};
pub use system_prompt::get_coordinator_system_prompt;
pub use types::{
    build_worker_tools_context, get_worker_tools, CoordinatorConfig, CoordinatorStatus, TaskNotification, TaskPhase,
    TaskStatus, TaskUsage, WorkerAgent, WorkerDirective, WorkerStatus, COORDINATOR_TOOLS, DEFAULT_WORKER_TOOLS,
    INTERNAL_ORCHESTRATION_TOOLS, SIMPLE_WORKER_TOOLS,
};
pub use worker_agent::{create_worker_agent, get_worker_system_prompt, is_worker_tool_available};
