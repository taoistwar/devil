# 协调器模式设计文档

## 概述

协调器模式将 Agent 变为编排者角色。编排者不直接操作文件，而是通过 Agent 工具派发任务给多个 worker 并行执行。适用于大型任务拆分、并行研究、实现 + 验证分离等场景。

## 核心约束

| 角色 | 可用工具 | 职责 |
|------|---------|------|
| **编排者** | Agent、SendMessage、TaskStop | 派发任务、综合结果、与用户沟通 |
| **Worker** | 所有标准工具 + MCP + Skill | 研究、实现、验证 |

- 编排者的每条消息都是给用户看的
- Worker 结果以 `<task-notification>` XML 形式到达
- Worker 不可见编排者对话，每个 prompt 必须自包含

## 启用方式

```bash
# 基本启用
FEATURE_COORDINATOR_MODE=1 CLAUDE_CODE_COORDINATOR_MODE=1 bun run dev

# 配合 Fork Subagent
FEATURE_COORDINATOR_MODE=1 FEATURE_FORK_SUBAGENT=1 \
CLAUDE_CODE_COORDINATOR_MODE=1 bun run dev

# Simple 模式（worker 只有 Bash/Read/Edit）
FEATURE_COORDINATOR_MODE=1 CLAUDE_CODE_COORDINATOR_MODE=1 \
CLAUDE_CODE_SIMPLE=1 bun run dev
```

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                        用户消息                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                    协调者 REPL                               │
│             (受限工具集：Agent/SendMessage/TaskStop)          │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
    ┌───────────────────┐           ┌───────────────────┐
    │  Agent({          │           │  SendMessage({    │
    │    subagent_type: │           │    to: "agent-id",│
    │    "worker",      │           │    message: "..." │
    │    prompt: "..."  │           │  })               │
    │  })               │           │                   │
    │                   │           │                   │
    │  派发新 Worker     │           │  继续现有 Worker   │
    └───────────────────┘           └───────────────────┘
              │                               │
              ▼                               ▼
    ┌─────────────────────────────────────────────────────┐
    │              Worker Agent（完整工具集）               │
    │    ├── 执行任务（Bash/Read/Edit/...）                │
    │    └── 返回 <task-notification>                       │
    └─────────────────────────────────────────────────────┘
                              │
                              ▼
    ┌─────────────────────────────────────────────────────┐
    │         <task-notification> XML 结果返回              │
    │  <task-id>agent-a1b</task-id>                        │
    │  <status>completed</status>                          │
    │  <summary>Agent completed</summary>                  │
    │  <result>Found null pointer...</result>              │
    └─────────────────────────────────────────────────────┘
```

## 任务阶段

| 阶段 | 执行者 | 目的 | 并发策略 |
|------|-------|------|---------|
| **Research** | Workers (并行) | 调查代码库，查找文件，理解问题 | 并行派发多个角度 |
| **Synthesis** | **协调者** | 阅读发现，理解问题，编写实现规范 | 串行（协调者执行） |
| **Implementation** | Workers | 根据规范进行有针对性的变更，提交 | 每套文件一个 Worker |
| **Verification** | Workers | 测试变更是否有效 | 可与实现并行（不同文件） |

## 核心设计决策

### 1. 双开关设计

| 开关 | 作用 |
|------|------|
| **Feature Flag** | 控制代码可用性（编译时） |
| **环境变量** | 控制实际激活（运行时） |

允许编译时包含但不默认启用。

### 2. 编排者受限

编排者只能用 `Agent`/`SendMessage`/`TaskStop`，确保专注于派发而非执行。

### 3. Worker 不可见编排者对话

每个 Worker 的 prompt 必须自包含所有必要上下文。Worker 无法看到：
- 编排者与用户的对话
- 其他 Worker 的结果（除非协调者综合后写入 prompt）

### 4. 并行优先

系统提示强调"Parallelism is your superpower"：
- Research 阶段：并行派发多个角度
- Implementation 阶段：不同文件集可并行
- Verification 阶段：可与实现并行（不同文件）

### 5. 综合而非转发

协调者必须理解 Worker 发现，再写出具体的实现指令。

**坏示例（懒惰委托）**：
```
Agent({ prompt: "Based on your findings, fix the auth bug" })
```

**好示例（综合后的规范）**：
```
Agent({ prompt: "Fix the null pointer in src/auth/validate.ts:42. The user field on Session (src/auth/types.ts:15) is undefined when sessions expire but the token remains cached. Add a null check before user.id access — if null, return 401 with 'Session expired'. Commit and report the hash." })
```

### 6. Scratchpad 可选共享

通过 GrowthBook 门控的共享目录，让 Worker 之间持久化共享知识：
```
Scratchpad directory: /tmp/scratchpad
Workers can read and write here without permission prompts.
```

## 继续 vs. 派发决策

| 情况 | 机制 | 原因 |
|------|------|------|
| 研究正好探索了需要编辑的文件 | **Continue** (SendMessage) + 综合规范 | Worker 已经有文件在上下文中 |
| 研究很广泛但实现很窄 | **Spawn fresh** (Agent) + 综合规范 | 避免探索噪音 |
| 纠正失败或扩展最近工作 | **Continue** | Worker 有错误上下文 |
| 验证另一个 Worker 刚写的代码 | **Spawn fresh** | 新鲜视角，无实现假设 |
| 第一次实现用错了方法 | **Spawn fresh** | 干净 slate，避免锚定失败 |
| 完全无关的任务 | **Spawn fresh** | 无有用上下文 |

## 任务通知格式

Worker 结果通过 `<task-notification>` XML 返回：

```xml
<task-notification>
  <task-id>agent-a1b</task-id>
  <status>completed|failed|killed</status>
  <summary>人类可读的状态摘要</summary>
  <result>Worker 的最终文本响应（可选）</result>
  <usage>
    <total_tokens>1234</total_tokens>
    <tool_uses>15</tool_uses>
    <duration_ms>5678</duration_ms>
  </usage>
</task-notification>
```

## Worker 工具集

### 默认模式

```
Bash, Read, Edit, Write, MultiEdit, NotebookEdit,
Glob, Grep, LS, TodoRead, TodoWrite,
+ MCP tools from configured MCP servers
+ Skill tool (project skills)
```

### Simple 模式

```
Bash, Read, Edit
```

### 禁用工具（仅协调者可用）

```
TeamCreate, TeamDelete, SendMessage, SyntheticOutput
```

## 典型工作流

```
用户："修复 auth 模块的 null pointer"

协调者:
  让我开始研究这个问题。

  Agent({ description: "调查 auth bug", subagent_type: "worker", prompt: "..." })
  Agent({ description: "研究 auth 测试", subagent_type: "worker", prompt: "..." })

  我正在并行调查这两个问题 —— 稍后报告发现。

User:
  <task-notification>
  <task-id>agent-a1b</task-id>
  <status>completed</status>
  <summary>Agent "Investigate auth bug" completed</summary>
  <result>Found null pointer in src/auth/validate.ts:42...</result>
  </task-notification>

协调者:
  找到了 bug —— src/auth/validate.ts:42 的空指针问题。我会修复它。
  还在等待 token 存储研究的結果。

  SendMessage({ to: "agent-a1b", message: "Fix the null pointer in src/auth/validate.ts:42..." })
```

## 实现架构

### 模式检测

```rust
pub fn is_coordinator_mode(config: &CoordinatorConfig) -> bool {
    config.enabled && is_env_coordinator_mode()
}
```

需要同时满足：
1. `CoordinatorConfig.enabled = true`
2. `CLAUDE_CODE_COORDINATOR_MODE=1` 环境变量

### 会话恢复

```rust
pub fn match_session_mode(
    stored_mode: Option<SessionMode>,
) -> Option<String>
```

恢复旧会话时检查存储的模式，自动翻转环境变量以匹配。

### 系统提示生成

协调者系统提示包含 6 个章节：
1. Your Role - 职责定义
2. Your Tools - 工具使用说明
3. Workers - Worker 能力描述
4. Task Workflow - 四阶段流程
5. Writing Worker Prompts - Prompt 编写指南
6. Example Session - 完整示例

### Worker Agent 定义

```rust
pub fn create_worker_agent(config: &CoordinatorConfig) -> WorkerAgent {
    WorkerAgent {
        agent_type: "worker".to_string(),
        when_to_use: "...",
        tools: get_worker_tools(config),
        system_prompt: get_worker_system_prompt().to_string(),
    }
}
```

### 任务编排器

```rust
pub struct Orchestrator {
    running_tasks: Arc<RwLock<Vec<RunningTask>>>,
}

impl Orchestrator {
    pub async fn spawn_research(&self, desc: impl Into<String>, prompt: impl Into<String>) -> String;
    pub async fn spawn_implementation(&self, desc: impl Into<String>, prompt: impl Into<String>) -> String;
    pub async fn spawn_verification(&self, desc: impl Into<String>, prompt: impl Into<String>) -> String;
    pub async fn continue_task(&self, task_id: &str, message: impl Into<String>) -> Result<(), String>;
    pub async fn stop_task(&self, task_id: &str) -> Result<(), String>;
    pub async fn synthesize_research(&self, findings: Vec<TaskNotification>) -> String;
}
```

## 使用示例

### 基本使用

```rust
use agent::coordinator::{Orchestrator, CoordinatorConfig};

let config = CoordinatorConfig {
    enabled: true,
    simple_mode: false,
    scratchpad_dir: Some("/tmp/scratchpad".to_string()),
    mcp_servers: vec![],
};

let orchestrator = Orchestrator::new();

// 并行派发研究任务
let task1 = orchestrator.spawn_research(
    "调查 auth bug",
    "研究 src/auth/validate.ts 中的空指针问题",
).await;

let task2 = orchestrator.spawn_research(
    "研究 token 存储",
    "研究安全的 token 存储方案",
).await;

println!("派发了两个任务：{} 和 {}", task1, task2);
```

### Prompt 构建器

```rust
use agent::coordinator::PromptBuilder;

let directive = PromptBuilder::new("修复 auth bug")
    .with_file_location("src/auth/validate.ts", Some(42), "存在空指针问题")
    .with_action("添加空值检查：if user.is_none() {{ return Err(401); }}")
    .with_purpose("修复用户登录问题")
    .with_report_requirement("提交后报告 commit hash")
    .build();
```

### 继续 vs. 派发决策

```rust
use agent::coordinator::{Orchestrator, ContinueOrSpawn, TaskPhase};

let orchestrator = Orchestrator::new();

let previous_task = RunningTask {
    task_id: "agent-123".to_string(),
    description: "Research".to_string(),
    phase: TaskPhase::Research,
    directive: ...,
};

// 研究完成后实现，继续
match orchestrator.should_continue_or_spawn(
    Some(&previous_task),
    "implement the fix",
) {
    ContinueOrSpawn::Continue => {
        // 使用 SendMessage 继续
    }
    ContinueOrSpawn::Spawn => {
        // 使用 Agent 派发新的
    }
}
```

## 关键设计决策

| 决策 | 说明 |
|------|------|
| **Fork ≠ 协调器** | Fork 继承上下文，协调器派发独立 Worker |
| **Coordindator 互斥 Fork** | Coordinator 模式下禁用 fork，两者有不兼容的委派模型 |
| **RenderedSystemPrompt 直传** | 避免 fork 时重新调用 `getSystemPrompt()` |
| **非交互式禁用** | pipe 模式和 SDK 模式下禁用，避免不可见的 fork 嵌套 |

## 文件索引

| 文件 | 职责 |
|------|------|
| `crates/agent/src/coordinator/mod.rs` | 模块入口和文档 |
| `crates/agent/src/coordinator/types.rs` | 类型定义 |
| `crates/agent/src/coordinator/mode_detection.rs` | 模式检测和会话恢复 |
| `crates/agent/src/coordinator/system_prompt.rs` | 系统提示生成 |
| `crates/agent/src/coordinator/worker_agent.rs` | Worker Agent 定义 |
| `crates/agent/src/coordinator/orchestration.rs` | 任务编排逻辑 |

## 与 Claude Code 对齐

| Claude Code 文件 | 本实现 |
|-----------------|--------|
| `src/coordinator/coordinatorMode.ts` | `mode_detection.rs`, `system_prompt.rs` |
| `src/coordinator/workerAgent.ts` | `worker_agent.rs` |
| `docs/features/coordinator-mode.md` | 本文档 |

## 待办事项

- [ ] 集成到 Agent 主循环
- [ ] 实现实际的 Worker 执行（与 Subagent 执行器集成）
- [ ] 实现 Scratchpad 目录管理
- [ ] 实现 MCP 工具集成
- [ ] 编写单元测试
- [ ] 编写集成测试
