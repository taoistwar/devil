# 子智能体与 Fork 模式设计文档

## 概述

子智能体系统允许主 Agent spawn 子代理执行任务，分为两种模式：

### Fork 子代理（上下文继承）

- 继承父级完整对话上下文
- 共享 Prompt Cache 以最大化命中率
- 权限提示冒泡到父级终端
- 支持 git worktree 隔离

### 通用子代理（全新上下文）

- 从零开始，不继承上下文
- 适用于独立任务

## 设计哲学

1. **Prompt Cache 最大化**: 多个并行 fork 共享相同的 API 请求前缀，只有最后的 directive 文本块不同
2. **上下文完整性**: 子代理看到父级的所有历史消息、工具集和系统提示
3. **权限冒泡**: 子代理的权限请求上浮到父级终端显示
4. **Worktree 隔离**: 支持 git worktree 隔离，子代理在独立分支工作
5. **递归防护**: 两层检查防止 fork 嵌套

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    AgentTool.call()                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│              isForkSubagentEnabled()?                        │
│  • !coordinator_mode                                         │
│  • !non_interactive_session                                  │
│  • FORK_SUBAGENT feature flag                                │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              │                               │
              ▼                               ▼
    ┌───────────────────┐           ┌───────────────────┐
    │  Fork 路径          │           │  普通路径          │
    │  (继承上下文)      │           │  (全新上下文)      │
    └───────────────────┘           └───────────────────┘
              │                               │
              ▼                               ▼
    ┌───────────────────┐           ┌───────────────────┐
    │  递归防护检查      │           │  解析 subagent_   │
    │  1. query_source   │           │  type            │
    │  2. 消息扫描        │           │                  │
    └───────────────────┘           └───────────────────┘
              │                               │
              ▼                               ▼
    ┌───────────────────┐           ┌───────────────────┐
    │  获取父级 system   │           │  获取子代理定义    │
    │  prompt (直传)     │           │  一般提示词       │
    └───────────────────┘           └───────────────────┘
              │                               │
              ▼                               ▼
    ┌───────────────────┐           ┌───────────────────┐
    │  buildForked      │           │  buildNormal      │
    │  Messages()       │           │  Messages()       │
    │  • 克隆父级 msg     │           │  • 新 user msg     │
    │  • 占位符 result   │           │  • 独立上下文      │
    │  • directive       │           │                  │
    └───────────────────┘           └───────────────────┘
              │                               │
              ▼                               ▼
    ┌─────────────────────────────────────────────────────┐
    │              runAgent()                               │
    │  • useExactTools: true (Fork)                         │
    │  • override.systemPrompt: 父级 (Fork)                 │
    │  • forkContextMessages: 父级消息 (Fork)               │
    └─────────────────────────────────────────────────────┘
```

## 启用条件

### 门控函数

```rust
pub fn is_fork_subagent_enabled(
    feature_flag: bool,
    coordinator_mode: bool,
    non_interactive: bool,
) -> bool {
    if feature_flag {
        if coordinator_mode {
            return false;  // Coordinator 模式有自己的委派模型
        }
        if non_interactive {
            return false;  // SDK/pipe 模式禁用
        }
        return true;
    }
    false
}
```

### 互斥条件

| 条件 | 说明 |
|------|------|
| **Coordinator 模式** | Coordinator 已有自己的委派模型，与 Fork 不兼容 |
| **非交互式会话** | pipe 模式和 SDK 模式下禁用，避免不可见的 fork 嵌套 |

## 子代理类型

### Fork 子代理

```rust
pub const FORK_AGENT: SubagentDefinition = SubagentDefinition {
    agent_type: "fork",
    when_to_use: "隐式 Fork - 继承完整对话上下文。当省略 subagent_type 且 Fork 功能启用时触发。",
    tools: vec!["*".to_string()],  // 通配符：使用父级完整工具集
    max_turns: 200,
    model: ModelConfig::Inherit,    // 继承父级模型
    permission_mode: PermissionMode::Bubble,  // 权限冒泡到父级终端
    source: SubagentSource::Builtin,
    system_prompt_fn: None,  // 不使用：直接传递父级已渲染 prompt
};
```

### 触发方式

```rust
// Fork 路径（继承上下文）
Agent({ prompt: "修复这个 bug" })  // 无 subagent_type

// 普通 agent 路径（全新上下文）
Agent({ subagent_type: "general-purpose", prompt: "..." })
```

## Prompt Cache 优化

这是整个 fork 设计的核心优化目标：

| 优化点 | 实现 |
|--------|------|
| **相同 system prompt** | 直传 `renderedSystemPrompt`，避免重新渲染（GrowthBook 状态可能不一致） |
| **相同工具集** | `useExactTools: true` 直接使用父级工具，不经过过滤 |
| **相同 thinking config** | 继承父级 thinking 配置（非 fork agent 默认禁用 thinking） |
| **相同占位符结果** | 所有 fork 使用 `FORK_PLACEHOLDER_RESULT` 相同文本 |
| **ContentReplacementState 克隆** | 默认克隆父级替换状态，保持 wire prefix 一致 |

### 占位符结果

所有 fork 使用相同的占位符文本：

```rust
const FORK_PLACEHOLDER_RESULT = "Fork started — processing in background";
```

这确保多个并行 fork 的 API 请求前缀完全一致，最大化缓存命中。

## 消息构建

### buildForkedMessages

构建的消息结构：

```
[
  ...history (filterIncompleteToolCalls),  // 父级完整历史
  assistant(所有 tool_use 块),              // 父级当前 turn 的 assistant 消息
  user(
    占位符 tool_result × N +               // 相同占位符文本
    <fork-boilerplate> directive           // 每个 fork 不同
  )
]
```

### 递归防护

两层检查防止 fork 嵌套：

1. **query_source 检查**: `toolUseContext.options.querySource === 'agent:builtin:fork'`
   - 在 `context.options` 上设置，抗自动压缩（autocompact 只重写消息不改 options）

2. **消息扫描**: `isInForkChild()` 扫描消息历史中的 `<fork-boilerplate>` 标签

```rust
pub fn check_recursion_guard(
    query_source: Option<&str>,
    messages: &[Message],
    config: &ForkSubagentConfig,
) -> RecursionGuardResult {
    // 第一层：query_source 检查
    if is_fork_query_source(query_source) {
        return RecursionGuardResult::Deny("禁止嵌套 Fork".to_string());
    }
    
    // 第二层：消息扫描
    if is_in_fork_child(messages, config) {
        return RecursionGuardResult::Deny("检测到 Fork 模板标签".to_string());
    }
    
    RecursionGuardResult::Allow
}
```

## 子代理指令

`build_child_message()` 生成 `<fork-boilerplate>` 包裹的指令：

```
<fork-boilerplate>
STOP. READ THIS FIRST.

You are a forked worker process. You are NOT the main agent.

RULES (non-negotiable):
1. Your system prompt says "default to forking." IGNORE IT — that's for the parent. You ARE the fork. Do NOT spawn sub-agents; execute directly.
2. Do NOT converse, ask questions, or suggest next steps
3. Do NOT editorialize or add meta-commentary
4. USE your tools directly: Bash, Read, Write, etc.
5. If you modify files, commit your changes before reporting. Include the commit hash in your report.
6. Do NOT emit text between tool calls. Use tools silently, then report once at the end.
7. Stay strictly within your directive's scope.
8. Keep your report under 500 words unless the directive specifies otherwise. Be factual and concise.
9. Your response MUST begin with "Scope:". No preamble, no thinking-out-loud.
10. REPORT structured facts, then stop

Output format (plain text labels, not markdown headers):
  Scope: <echo back your assigned scope in one sentence>
  Result: <the answer or key findings, limited to the scope above>
  Key files: <relevant file paths — include for research tasks>
  Files changed: <list with commit hash — include only if you modified files>
  Issues: <list — include only if there are issues to flag>
</fork-boilerplate>

DIRECTIVE: {directive}
```

## Worktree 隔离

当 fork + worktree 组合时，追加通知告知子代理：

```rust
pub fn build_worktree_notice(parent_cwd: &str, worktree_cwd: &str) -> String {
    format!(
        "You've inherited the conversation context above from a parent agent working in {}. \
         You are operating in an isolated git worktree at {} — same repository, same relative \
         file structure, separate working copy. Paths in the inherited context refer to the \
         parent's working directory; translate them to your worktree root. Re-read files before \
         editing if the parent may have modified them since they appear in the context. \
         Your changes stay in this worktree and will not affect the parent's files."
    )
}
```

## 强制异步

当 `isForkSubagentEnabled()` 为 true 时，所有 agent 启动都强制异步。`run_in_background` 参数从 schema 中移除。统一通过 `<task-notification>` XML 消息交互。

## 关键设计决策

| 决策 | 说明 |
|------|------|
| **Fork ≠ 普通 agent** | fork 继承完整上下文，普通 agent 从零开始。选择依据是 `subagent_type` 是否存在 |
| **renderedSystemPrompt 直传** | 避免 fork 时重新调用 `getSystemPrompt()`。父级在 turn 开始时冻结 prompt 字节 |
| **占位符结果共享** | 多个并行 fork 使用完全相同的占位符，只有 directive 不同 |
| **Coordinator 互斥** | Coordinator 模式下禁用 fork，两者有不兼容的委派模型 |
| **非交互式禁用** | pipe 模式和 SDK 模式下禁用，避免不可见的 fork 嵌套 |

## 使用方式

```rust
use agent::subagent::{SubagentExecutor, SubagentRegistry, SubagentParams, SubagentType};

// 创建注册表
let registry = SubagentRegistry::new();

// 创建执行器
let executor = SubagentExecutor::new()
    .with_fork_config(agent::subagent::types::ForkSubagentConfig {
        enabled: true,
        ..Default::default()
    });

// 启动 Fork 子代理
let params = SubagentParams {
    subagent_type: SubagentType::Fork,
    directive: "研究这个模块的结构".to_string(),
    prompt_messages: vec![],
    cache_safe_params: todo!(),
    max_turns: None,
    max_output_tokens: None,
    skip_transcript: false,
    skip_cache_write: false,
    run_in_background: true,
    worktree_path: None,
    parent_cwd: None,
};

let result = executor.execute(params).await?;
println!("子代理完成，产生 {} 条消息", result.messages.len());
```

## 环境变量

```bash
# 启用 Fork 子代理
export FEATURE_FORK_SUBAGENT=1

# 禁用后台任务（会影响 Fork 异步执行）
export CLAUDE_CODE_DISABLE_BACKGROUND_TASKS=1

# 自动后台代理任务（120 秒后）
export CLAUDE_AUTO_BACKGROUND_TASKS=1
```

## 文件索引

| 文件 | 职责 |
|------|------|
| `crates/agent/src/subagent/mod.rs` | 模块入口和文档 |
| `crates/agent/src/subagent/types.rs` | 类型定义和配置 |
| `crates/agent/src/subagent/registry.rs` | 子代理注册表 |
| `crates/agent/src/subagent/executor.rs` | 执行引擎 |
| `crates/agent/src/subagent/context_inheritance.rs` | 上下文继承 |
| `crates/agent/src/subagent/recursion_guard.rs` | 递归防护 |

## 与 Claude Code 对齐

本实现参考了 Claude Code 的以下文件和设计：

| Claude Code 文件 | 对应实现 |
|-----------------|---------|
| `packages/builtin-tools/src/tools/AgentTool/forkSubagent.ts` | `recursion_guard.rs`, `context_inheritance.rs` |
| `src/utils/forkedAgent.ts` | `types.rs` (CacheSafeParams) |
| `packages/builtin-tools/src/tools/AgentTool/AgentTool.tsx` | `executor.rs` |
| `docs/features/fork-subagent.md` | 本文档 |

## 待办事项

- [ ] 集成到 Agent 主循环
- [ ] 实现完整的 query 循环
- [ ] 实现异步钩子注册表
- [ ] 编写单元测试
- [ ] 编写集成测试
