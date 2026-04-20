# Multi-Agent Coordinator Quickstart

## 概述

多 Agent 协调器模式允许主 Agent 作为"协调者"，通过派发 Worker Agent 并行执行复杂任务。

## 核心概念

| 概念 | 说明 |
|------|------|
| **Coordinator** | 主 Agent，负责任务分解、派发和结果综合 |
| **Worker** | 子 Agent，拥有受限工具集，执行具体任务 |
| **SubAgent** | Worker 派生的子 Agent，用于处理嵌套复杂度 |

## 启用协调器模式

```rust
use agent::coordinator::{enable_coordinator_mode, CoordinatorConfig};

let config = CoordinatorConfig {
    enabled: true,
    simple_mode: false,
    scratchpad_dir: Some("/tmp/scratchpad".to_string()),
    mcp_servers: vec!["github".to_string()],
};

enable_coordinator_mode(config);
```

## 派发任务

```rust
use agent::coordinator::{Orchestrator, WorkerDirective};

// 创建编排器
let orchestrator = Orchestrator::new();

// 并行派发研究任务
let task1 = orchestrator.spawn_research(
    "调查 auth bug",
    "研究 src/auth/ 模块中的空指针问题..."
).await;

let task2 = orchestrator.spawn_research(
    "研究 auth 测试",
    "查找所有 auth 相关的测试文件..."
).await;
```

## 继续 vs 派发决策

```rust
use agent::coordinator::{ContinueOrSpawn, RunningTask};

let decision = orchestrator.should_continue_or_spawn(
    Some(&previous_task),
    "implement the fix"
);

match decision {
    ContinueOrSpawn::Continue => {
        // 使用 SendMessage 继续现有 Worker
    }
    ContinueOrSpawn::Spawn => {
        // 使用 Agent 派发新 Worker
    }
}
```

## Worker 工具限制

Worker 默认工具集：`Bash`, `Read`, `Edit`, `Write`, `MultiEdit`, `Glob`, `Grep`, `LS`, `TodoRead`, `TodoWrite`

Simple 模式：`Bash`, `Read`, `Edit`

禁止使用：`TeamCreate`, `TeamDelete`, `SendMessage`, `SyntheticOutput`

## 任务通知格式

Worker 完成时返回 `TaskNotification`：

```rust
use agent::coordinator::{TaskNotification, TaskStatus};

let notification = TaskNotification::completed(
    "agent-123",
    "调查完成",
    Some("在 validate.ts:42 发现 null pointer".to_string())
);
```

## 子 Agent 嵌套

```rust
use agent::subagent::{SubagentExecutor, SubagentParams, SubagentType};

let params = SubagentParams {
    subagent_type: SubagentType::Fork,
    directive: "子任务指令...".to_string(),
    // ...
};

let result = executor.execute(params).await?;
```

## 最佳实践

1. **并行优先**：独立任务并行派发
2. **自包含 Prompt**：Worker 看不到协调者对话，Prompt 必须包含所有必要上下文
3. **综合而非转发**：协调者必须理解 Worker 发现后再写出具体指令
4. **深度限制**：子 Agent 嵌套最多 3 层

## 常见问题

**Q: Worker 可以访问哪些工具？**  
A: 默认是 Bash、Read、Edit 及标准工具。Simple 模式下仅 Bash/Read/Edit。

**Q: 如何停止失控的 Worker？**  
A: 使用 `orchestrator.stop_task(task_id)` 停止任务。

**Q: 子 Agent 最多嵌套几层？**  
A: 默认 3 层，通过 `RecursionGuard` 强制执行。
