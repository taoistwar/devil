# Implementation Plan: Claude Code Memory System Alignment

**Branch**: `260419-feat-claude-code-context-alignment` | **Date**: 2026-04-19 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-claude-code-memory-alignment/spec.md`

## Summary

Implement a persistent, file-based memory system aligned with Claude Code's memory.ts. The system organizes memories by type (user, feedback, project, reference) in a memory directory with MEMORY.md as the index entrypoint. Memory files use frontmatter format with truncation at 200 lines / 25KB.

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: `tokio`, `anyhow`, `serde`/`serde_yaml` for frontmatter parsing  
**Storage**: File-based (memory directory at `~/.claude/projects/<project>/memory/`)  
**Testing**: `cargo test`, integration tests for memory operations  
**Target Platform**: Linux server environments  
**Project Type**: CLI tool / Agent library (`devil-agent`)  
**Performance Goals**: Memory file reads <10ms for typical files  
**Constraints**: Support `CLAUDE_CODE_DISABLE_AUTO_MEMORY` env var, bare mode disabled  
**Scale/Scope**: Single-user memory, ~100 memory files per project

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| I. Rust-First Standards | PASS | Idiomatic Rust, ownership model, no unsafe |
| II. Tokio Concurrency | PASS | All async via Tokio, `#[async_trait]` |
| III. Claude Code Parity | PASS | Following reference memory.ts patterns |
| IV. Robust Error Handling | PASS | `anyhow::Result` with context |
| V. Tool-First Architecture | PASS | Memory system integrates with existing tool framework |

## Project Structure

### Documentation (this feature)

```text
specs/007-claude-code-memory-alignment/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (if needed)
└── tasks.md             # Phase 2 output
```

### Source Code (repository root)

```text
crates/agent/src/
├── context/
│   ├── mod.rs              # Updated exports
│   ├── memory_files.rs      # [EXISTING] CLAUDE.md discovery
│   └── memory/              # [NEW] Full memory system
│       ├── mod.rs           # Memory module entry
│       ├── types.rs         # MemoryType, MemoryEntry, frontmatter
│       ├── dir.rs           # MemoryDir path resolution
│       ├── index.rs         # MEMORY.md index management
│       ├── truncation.rs     # Line/byte truncation
│       └── prompts.rs        # Memory guidance text
└── commands/
    └── advanced/
        └── memory.rs         # [EXISTING] /memory command stub

crates/agent/src/tools/enhanced/
└── memory/                   # [NEW] Memory operation tools
    ├── mod.rs
    ├── save.rs               # SaveMemory tool
    ├── recall.rs              # RecallMemory tool
    └── forget.rs              # ForgetMemory tool
```

**Structure Decision**: Add new `memory/` subdirectory under `context/` for memory system modules. Add memory tools under `tools/enhanced/memory/`. Extend existing `memory_files.rs` for CLAUDE.md discovery.

## Phase 0: Research

### Findings from Claude Code Reference

**Decision**: Implement memory system following Claude Code's typed-memory pattern
**Rationale**: Proven design with eval-validated prompting guidance
**Alternatives considered**: Simple key-value store, vector embedding retrieval

### Key Implementation Insights

1. **Memory Types**: Four types (user, feedback, project, reference) with frontmatter
2. **Index Pattern**: MEMORY.md is index only, topic files hold content
3. **Truncation**: Line-first (200 lines), then byte (25KB) at last newline
4. **Guidance**: Include what NOT to save in memory prompts
5. **Verification**: Agent should verify memory accuracy before acting

## Phase 1: Design

### Data Model

See `data-model.md` for complete entity definitions.

### Memory Directory Structure

```
~/.claude/projects/<sanitized-project>/memory/
├── MEMORY.md           # Index file (entrypoint)
├── user_*.md          # User type memories
├── feedback_*.md       # Feedback memories
├── project_*.md        # Project memories
└── reference_*.md      # Reference memories
```

### Frontmatter Format

```yaml
---
name: memory-name
description: one-line description for relevance matching
type: user|feedback|project|reference
---
Memory content with optional Why:/How to apply: sections
```

## Complexity Tracking

> No constitution violations requiring justification.