# Implementation Plan: Claude Code Tools Alignment

**Branch**: `260418-feat-claude-code-tools-alignment` | **Date**: 2026-04-18 | **Spec**: [spec.md](./spec.md)

## Summary

成功实现与 Claude Code 对齐的 10 个核心工具，包括 6 个原有工具（Bash, Read, Edit, Write, Glob, Grep）和 4 个新增工具（WebFetch, WebSearch, TodoWrite, Agent）。所有工具遵循 Rust-First 标准和 Tokio 并发模型。

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: tokio, reqwest, regex, glob, walkdir, async-trait, serde  
**Storage**: N/A (in-memory execution)  
**Testing**: cargo test, 单元测试覆盖各工具  
**Target Platform**: Linux server environments  
**Project Type**: CLI tool library (devil-agent)  
**Performance Goals**: 工具执行响应 < 100ms（不含实际命令执行）  
**Constraints**: 遵循 Claude Code 五要素协议 (Input, Output, Progress, Permissions, Metadata)  
**Scale/Scope**: 10 个工具，53 个 Claude Code 工具中的核心集合

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| Rust-First Standards | ✅ PASS | 所有代码使用 idiomatic Rust |
| Tokio Concurrency Model | ✅ PASS | 使用 `#[async_trait]` 和 Tokio runtime |
| Claude Code Reference Parity | ✅ PASS | 工具名称、参数与参考实现一致 |
| Robust Error Handling | ✅ PASS | 使用 `anyhow::Result<T>` 和 `thiserror` |
| Tool-First Architecture | ✅ PASS | 所有功能通过工具暴露 |

## Completed Implementation

### Phase 1: Core Tools (DONE)
| 工具 | 状态 | 实现文件 |
|------|------|----------|
| BashTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileReadTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileEditTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileWriteTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| GlobTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| GrepTool | ✅ | `crates/agent/src/tools/builtin.rs` |

### Phase 2: Web & Task Tools (DONE)
| 工具 | 状态 | 实现文件 |
|------|------|----------|
| WebFetchTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| WebSearchTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| TodoWriteTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| AgentTool | ✅ | `crates/agent/src/tools/builtin.rs` |

### Phase 3: Enhanced Features (DONE)
| 功能 | 状态 | 实现文件 |
|------|------|----------|
| WebFetch CSS 选择器 | ✅ | `crates/agent/src/tools/builtin.rs` |

## Implementation Details

### Tool Trait (五要素协议)

```rust
pub trait Tool: Send + Sync {
    type Input: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    type Output: Serialize + for<'de> Deserialize<'de> + Send + Sync;
    type Progress: ToolProgressData;

    fn name(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    async fn execute(&self, input: Self::Input, ctx: &ToolContext, ...) -> Result<ToolResult<Self::Output>>;
    // ... permission, timeout, metadata methods
}
```

### Tool Registry

工具在 `Agent::register_default_tools()` 中注册：

```rust
pub async fn register_default_tools(&self) -> Result<()> {
    use crate::tools::builtin::{
        AgentTool, BashTool, FileEditTool, FileReadTool, FileWriteTool, GlobTool, GrepTool,
        TodoWriteTool, WebFetchTool, WebSearchTool,
    };
    self.register_tool(BashTool::new(false)).await?;
    self.register_tool(FileReadTool::default()).await?;
    // ... 注册所有工具
}
```

### Permission Model

```rust
pub enum ToolPermissionLevel {
    ReadOnly,              // 只读，无需确认
    RequiresConfirmation,   // 需要确认
    Destructive,            // 破坏性操作，严格受限
    BlanketDenied,          // 任何情况都不允许
}
```

## Remaining Work

### Enhanced Capabilities (FR-101-FR-105)
| 功能 | 状态 | 优先级 |
|------|--------|--------|
| Bash 命令历史和自动补全 | 🔲 待实现 | P2 |
| Read 语法高亮标记 | 🔲 待实现 | P3 |
| Glob 排除模式 | 🔲 待实现 | P2 |
| 工具执行结果流式输出 | 🔲 待实现 | P3 |

### Testing
| 测试 | 状态 | 说明 |
|------|------|------|
| 单元测试覆盖 | 🔲 待完善 | 每个工具需 3 个测试用例 |
| 集成测试 | 🔲 待实现 | 工具链测试 |

## Project Structure

```text
crates/agent/src/
├── tools/
│   ├── builtin.rs          # 10 个工具实现
│   ├── tool.rs              # Tool trait 定义
│   ├── registry.rs          # 工具注册表
│   ├── executor.rs          # 工具执行器
│   └── build_tool.rs        # 工具构建器
├── subagent/
│   ├── mod.rs              # 子代理模块
│   ├── executor.rs          # 子代理执行器
│   └── types.rs            # 子代理类型定义
└── core.rs                 # Agent 主模块
```

## Complexity Tracking

> 无复杂度违规

## Next Steps (Spec 004)

Spec 004 (`specs/004-remaining-claude-code-tools/`) 涵盖剩余 43 个工具的实现：
- Phase 1: Task, Planning, Worktree, MCP tools
- Phase 2: Config, Skills, Scheduling, Workflow, Communication tools
- Phase 3: File, LSP, Enhanced tools
