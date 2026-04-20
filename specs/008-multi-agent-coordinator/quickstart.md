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

Worker 可以派发子 Agent 来处理嵌套复杂度：

```rust
use agent::coordinator::{Orchestrator, WorkerDirective, MAX_SUBAGENT_DEPTH};

// 检查是否可以派发子 Agent
if orchestrator.can_spawn_subagent(parent_depth) {
    let subagent_id = orchestrator.spawn_subagent(
        "agent-123",  // 父任务 ID
        WorkerDirective::research("子任务", "嵌套研究..."),
        agent::coordinator::types::TaskPhase::Research,
    ).await?;
}
```

### 深度跟踪

`RunningTask` 包含 `depth` 字段跟踪嵌套层级：

```rust
pub struct RunningTask {
    pub task_id: String,
    pub description: String,
    pub phase: TaskPhase,
    pub directive: WorkerDirective,
    pub depth: u8,  // 嵌套深度
}

const MAX_SUBAGENT_DEPTH: u8 = 3;
```

## 结果聚合

多个 Worker 完成时，使用 `aggregate_results()` 聚合：

```rust
use agent::coordinator::{TaskNotification, AggregatedResults};

// 收集 TaskNotification
let notifications = vec![
    TaskNotification::completed("agent-1", "完成", Some("结果1")),
    TaskNotification::completed("agent-2", "完成", Some("结果2")),
];

let results = orchestrator.aggregate_results(notifications).await;

println!("成功率: {:.1}%", results.success_rate() * 100.0);
println!("摘要: {}", results.summary());
```

### AggregatedResults

```rust
pub struct AggregatedResults {
    pub successful: Vec<TaskNotification>,  // 成功的任务
    pub failed: Vec<TaskNotification>,      // 失败的任务
    pub killed: Vec<TaskNotification>,     // 被杀的任务
    pub total_count: usize,                // 总数
}

impl AggregatedResults {
    pub fn all_succeeded(&self) -> bool;
    pub fn success_rate(&self) -> f64;
    pub fn summary(&self) -> String;
}
```

## 失败处理

Worker 失败时，`TaskFailureAction` 定义处理策略：

```rust
pub enum TaskFailureAction {
    /// 报告给用户
    ReportToUser {
        task_id: String,
        description: String,
        error: String,
    },
    /// 重新派发任务
    Reassign {
        original_task_id: String,
        reason: String,
    },
    /// 未知任务
    UnknownTask,
}

// 处理失败
let action = orchestrator.on_task_failed(notification).await;
match action {
    TaskFailureAction::Reassign { original_task_id, reason } => {
        orchestrator.reassign_task(new_directive).await?;
    }
    _ => {}
}
```

## 协调器状态

查看当前协调器状态：

```rust
use agent::coordinator::CoordinatorStatus;

let status = orchestrator.get_coordinator_status(
    is_coordinator_mode(&config),
    config.simple_mode
).await;

println!("启用: {}", status.enabled);
println!("活跃 Worker: {}", status.active_workers.len());
println!("Simple 模式: {}", status.simple_mode);

for worker in status.active_workers {
    println!("  - {} ({})", worker.task_id, worker.phase);
}
```

## 最佳实践

1. **并行优先**：独立任务并行派发
2. **自包含 Prompt**：Worker 看不到协调者对话，Prompt 必须包含所有必要上下文
3. **综合而非转发**：协调者必须理解 Worker 发现后再写出具体指令
4. **深度限制**：子 Agent 嵌套最多 3 层
5. **结果聚合**：使用 `aggregate_results()` 统一处理多个 Worker 结果

## 常见问题

**Q: Worker 可以访问哪些工具？**  
A: 默认是 Bash、Read、Edit 及标准工具。Simple 模式下仅 Bash/Read/Edit。

**Q: 如何停止失控的 Worker？**  
A: 使用 `orchestrator.stop_task(task_id)` 停止任务。

**Q: 子 Agent 最多嵌套几层？**  
A: 默认 3 层，通过 `MAX_SUBAGENT_DEPTH` 常量强制执行。

**Q: Worker 失败后如何处理？**  
A: 使用 `on_task_failed()` 获取失败动作，可选择重新派发或报告用户。
