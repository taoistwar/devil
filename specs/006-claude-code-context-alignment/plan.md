# Implementation Plan: Claude Code Context Alignment (Spec 006)

**Branch**: `006-claude-code-context-alignment` | **Date**: 2026-04-19 | **Spec**: [spec.md](../spec.md)

## Summary

实现 Claude Code 上下文注入功能对齐。需要在对话开始时收集并注入 Git 状态、当前日期和 Memory 文件（CLAUDE.md）内容到系统/用户上下文。

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: tokio, anyhow, serde, memoize (custom)
**Storage**: File system (TOML for config)
**Testing**: cargo test
**Target Platform**: Linux/macOS/Windows CLI
**Project Type**: CLI tool with context system
**Performance Goals**: 上下文收集 < 100ms，缓存命中时 < 1ms
**Constraints**: 非阻塞 I/O，缓存避免重复收集

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| Rust-First Standards | ✅ | All code idiomatic Rust |
| Tokio Concurrency | ✅ | All async use tokio |
| Claude Code Parity | ✅ | Aligned with context.ts |
| Robust Error Handling | ✅ | anyhow::Result throughout |
| Tool-First Architecture | ✅ | CLI tools exposed |

## Gates

| Gate | Status | Justification |
|------|--------|---------------|
| cargo build success | ✅ | Build must pass |
| cargo clippy clean | ⚠️ | 26 warnings remain (type complexity) |
| cargo test pass | ✅ | Tests must pass |
| Constitution compliance | ✅ | All principles followed |

## Project Structure

### Documentation (this feature)

```text
specs/006-claude-code-context-alignment/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0 output (skipped - no unknowns)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── checklists/
    └── requirements.md  # Quality checklist
```

### Source Code

```text
crates/agent/src/
├── context/
│   ├── mod.rs                      # Existing context module
│   ├── git_status.rs                # NEW: Git status collection
│   ├── memory_files.rs              # NEW: CLAUDE.md discovery & loading
│   ├── system_context.rs            # NEW: System context (git + injection)
│   └── user_context.rs              # NEW: User context (memory + date)

crates/agent/src/tools/
└── builtin.rs                      # May need context tools
```

## Key Design Decisions

### 1. Context Collection Strategy

**Decision**: Lazy + memoized collection
**Rationale**: Git/file operations are I/O heavy; cache results for conversation duration
**Alternatives**: Eager (wastes resources), Real-time (too expensive)

### 2. Git Status Format

**Decision**: Match Claude Code format exactly
**Rationale**: Parity requirement - AI expects specific format
**Format**:
```
Current branch: {branch}
Main branch: {mainBranch}
Git user: {userName}
Status:
{status or '(clean)'}
Recent commits:
{log}
```

### 3. Memory Files Discovery

**Decision**: Recursive directory walk from CWD to root
**Rationale**: CLAUDE.md can be in any parent directory (monorepo support)
**Filter**: Skip node_modules, .git, and hidden directories

### 4. Cache Breaker Implementation

**Decision**: Module-level static with atomic swap
**Rationale**: Simple, effective, matches reference pattern

## Implementation Phases

### Phase 1: Core Infrastructure

1. Create `GitStatusCollector` for git status collection
2. Create `MemoryFilesCollector` for CLAUDE.md discovery
3. Implement `SystemContextProvider` and `UserContextProvider`
4. Add memoization wrapper

### Phase 2: Integration

1. Integrate with existing `ContextManager`
2. Add environment variable support (CLAUDE_CODE_DISABLE_CLAUDE_MDS, CLAUDE_CODE_REMOTE)
3. Add bare mode support

### Phase 3: Testing

1. Unit tests for each collector
2. Integration tests for full pipeline
3. Edge case tests (non-git dir, empty files, truncation)

## Artifacts

- **Implementation**: `crates/agent/src/context/git_status.rs`, `memory_files.rs`, `system_context.rs`, `user_context.rs`
- **Tests**: Unit tests in each module
- **Integration**: With `crates/agent/src/context/mod.rs`

## Open Questions

None - all requirements resolved in spec phase.
