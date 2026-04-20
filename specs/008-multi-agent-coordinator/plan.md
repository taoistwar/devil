# Implementation Plan: Multi-Agent Coordinator

**Branch**: `260419-feat-multi-agent-coordinator` | **Date**: 2026-04-19 | **Spec**: [link](../spec.md)
**Input**: Feature specification from `/specs/008-multi-agent-coordinator/spec.md`

## Summary

实现多 Agent 协调器模式，主 Agent 作为协调者派发任务给 Worker Agent 并行执行。Worker 拥有受限工具集（Bash、Read、Edit、MCP），可进一步生成子 Agent。

**技术方案**：基于现有 `coordinator/` 和 `subagent/` 模块进行增强，实现与 Claude Code `coordinatorMode.ts` 的完整对齐。

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: `tokio`, `anyhow`, `serde`  
**Storage**: N/A  
**Testing**: `cargo test`, `cargo clippy`  
**Target Platform**: Linux server  
**Project Type**: CLI tool / Agent framework  
**Performance Goals**: 支持 4+ 并发 Worker，10 秒内检测 Worker 失败  
**Constraints**: Tokio 异步运行时，工具限制 100% 强制执行  
**Scale/Scope**: 多 Worker 并行，3 层子 Agent 嵌套  

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| Rust-First Standards | ✅ PASS | 使用 Rust + Tokio |
| Tokio Concurrency | ✅ PASS | 所有 async 使用 Tokio |
| Claude Code Parity | ⚠️ PARTIAL | 基础功能对齐，需增强部分 |
| Error Handling | ✅ PASS | 使用 anyhow/thiserror |
| Tool-First Architecture | ✅ PASS | 工具系统已实现 |

## Project Structure

### Documentation (this feature)

```
specs/008-multi-agent-coordinator/
├── plan.md              # This file
├── research.md          # Phase 0 output (NOT NEEDED - existing impl sufficient)
├── data-model.md        # Phase 1 output (NOT NEEDED - types.rs already defines)
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (NOT NEEDED - internal only)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```
crates/agent/src/
├── coordinator/         # Existing - needs enhancement
│   ├── mod.rs
│   ├── types.rs        # TaskNotification, WorkerDirective, CoordinatorConfig
│   ├── mode_detection.rs
│   ├── orchestration.rs # Orchestrator (needs TODOs resolved)
│   ├── system_prompt.rs
│   └── worker_agent.rs
├── subagent/            # Existing - needs enhancement
│   ├── mod.rs
│   ├── types.rs        # SubagentType, SubagentParams
│   ├── registry.rs     # SubagentRegistry
│   ├── executor.rs     # Execution engine
│   ├── context_inheritance.rs
│   └── recursion_guard.rs # Max depth enforcement
└── commands/            # CLI commands
    └── coordinator.rs   # NEW - /coordinator command
```

**Structure Decision**: 增强现有 `coordinator/` 和 `subagent/` 模块，添加 CLI 命令入口

## Phase 0: Research (Skipped)

现有实现已完整覆盖 Claude Code `coordinatorMode.ts` 的核心功能：
- ✅ `isCoordinatorMode()` → `mode_detection.rs`
- ✅ `getCoordinatorUserContext()` → `types.rs:build_worker_tools_context()`
- ✅ `getCoordinatorSystemPrompt()` → `system_prompt.rs`
- ✅ Worker tools restriction → `worker_agent.rs:is_worker_tool_available()`
- ✅ Task notification format → `types.rs:TaskNotification`
- ✅ Continue vs spawn decision → `orchestration.rs:should_continue_or_spawn()`

## Phase 1: Design

### Key Entities (already defined in types.rs)

| Entity | Location | Status |
|--------|----------|--------|
| CoordinatorConfig | `coordinator/types.rs` | ✅ Complete |
| TaskNotification | `coordinator/types.rs` | ✅ Complete |
| WorkerDirective | `coordinator/types.rs` | ✅ Complete |
| SubagentType | `subagent/types.rs` | ✅ Complete |
| SubagentParams | `subagent/types.rs` | ✅ Complete |
| RecursionGuardResult | `subagent/recursion_guard.rs` | ✅ Complete |

### Gap Analysis

| Gap | Impact | Resolution |
|-----|--------|------------|
| `orchestration.rs` TODOs | Medium | 实现真实的 Worker spawn/continue/stop |
| Worker → SubAgent spawn | High | 在 WorkerAgent 中集成 SubagentExecutor |
| `/coordinator` CLI command | Low | 添加命令入口 |
| Max depth enforcement | Low | 已通过 RecursionGuard 实现 |

## Phase 2: Implementation Tasks (via /speckit.tasks)

1. **T001**: Resolve `orchestration.rs` TODOs - 实现真实 Worker 管理
2. **T002**: Integrate SubagentExecutor into worker flow
3. **T003**: Add `/coordinator` CLI command
4. **T004**: Add integration tests for coordinator mode
5. **T005**: Verify clippy passes with zero warnings
6. **T006**: Run cargo test 100% pass

## Complexity Tracking

> No violations - existing implementation is well-structured
