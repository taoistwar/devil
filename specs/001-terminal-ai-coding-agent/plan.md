# Implementation Plan: Terminal AI Coding Agent

**Branch**: `260417-feat-terminal-ai-coding-agent` | **Date**: 2026-04-17 | **Spec**: [spec.md](./spec.md)
**Input**: Build a terminal-based AI coding agent similar to Claude Code with Rust technology stack

## Summary

Implement a terminal-based AI coding agent that enables developers to delegate complex coding tasks to an AI agent capable of interacting with their codebase. The agent uses Claude API for reasoning, executes shell commands safely, reads/writes files, and maintains an interactive dialogue with the user.

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: `tokio` (async runtime), `anyhow`/`thiserror` (error handling), `clap` (CLI), `tracing` (logging), `serde` (serialization)  
**Storage**: Session files in `~/.devil/sessions/`, config in `~/.devil/config.toml`  
**Testing**: `cargo test`, integration tests in `tests/`  
**Target Platform**: Linux/macOS terminal environments  
**Project Type**: CLI tool / terminal application with AI integration  
**Performance Goals**: Response latency < 2s for tool operations, graceful shutdown < 2 seconds  
**Constraints**: Must maintain Claude Code reference parity, tool semantics must match reference  
**Scale/Scope**: Single-user terminal sessions, session-based context management

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I. Rust-First Standards**: Implementation uses idiomatic Rust with `cargo clippy` cleanliness
- [x] **II. Tokio Concurrency Model**: Async operations use Tokio runtime with `#[tokio::main]`
- [x] **III. Claude Code Reference Parity**: Tool semantics (Bash, Read, Edit, Write, Glob, Grep) match reference
- [x] **IV. Robust Error Handling**: `anyhow`/`thiserror` for proper error propagation with context
- [x] **V. Tool-First Architecture**: CLI tools expose core functionality with text-in/out protocol

## Project Structure

### Documentation (this feature)

```
specs/001-terminal-ai-coding-agent/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output (CLI entrypoint alignment reused)
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md            # Phase 2 output
```

### Source Code (repository root)

```
src/
├── main.rs              # Entry point with version fast-path
├── cli/                 # CLI module (from spec 002)
│   ├── mod.rs
│   ├── dispatcher.rs
│   ├── commands/
│   ├── init.rs
│   └── error.rs
├── agent/               # Core agent library (NEW - reorg from current structure)
│   ├── src/
│   │   ├── lib.rs           # Agent library exports
│   │   ├── core.rs          # Main agent orchestration
│   │   ├── message.rs       # Message types
│   │   ├── tools/           # Tool implementations
│   │   │   ├── mod.rs
│   │   │   ├── bash.rs      # Bash tool
│   │   │   ├── read.rs      # Read tool
│   │   │   ├── write.rs     # Write tool
│   │   │   ├── edit.rs      # Edit tool
│   │   │   ├── glob.rs      # Glob tool
│   │   │   └── grep.rs      # Grep tool
│   │   ├── context/         # Agent context management
│   │   │   ├── mod.rs
│   │   │   ├── session.rs   # Session state
│   │   │   └── history.rs   # Message history
│   │   ├── subagent/        # Sub-agent handling
│   │   │   ├── mod.rs
│   │   │   └── executor.rs
│   │   ├── permissions/     # Permission checking
│   │   │   ├── mod.rs
│   │   │   └── pipeline.rs
│   │   ├── coordinator/     # Task coordination
│   │   │   └── orchestration.rs
│   │   ├── hooks/          # Extension hooks
│   │   │   ├── mod.rs
│   │   │   └── executor.rs
│   │   └── skills/         # Skill system
│   │       ├── mod.rs
│   │       ├── loader.rs
│   │       └── executor.rs
├── mcp/                 # MCP integration (from spec 002)
│   └── src/
├── streaming/           # Streaming infrastructure (from spec 002)
│   └── src/
├── memory/              # Memory subsystem
│   └── src/
├── plugins/            # Plugin system
│   └── src/
├── providers/          # LLM providers
│   └── src/
│       └── anthropic.rs
└── channels/           # IPC channels
    └── src/

crates/
├── agent/              # Main agent crate
├── mcp/                # MCP protocol crate
├── streaming/          # Streaming crate
├── memory/             # Memory crate
├── plugins/            # Plugin crate
├── providers/          # Providers crate
├── channels/           # Channels crate
└── devil-agent/        # Main binary crate

tests/
├── cli_version_test.rs
├── cli_help_test.rs
├── agent/
│   ├── tool_test.rs
│   ├── session_test.rs
│   └── permission_test.rs
└── integration/
    └── full_session_test.rs
```

**Structure Decision**: Reorganize from flat `agent/src/` structure to feature-based modules under `src/agent/` with separate crates for each major subsystem.

## Complexity Tracking

No complexity violations - this is a Rust monorepo following constitution guidelines.

## Phase 0: Research

- [x] CLI entrypoint alignment completed (spec 002)
- [x] Tool semantics research from references/claude-code
- [x] Async runtime patterns from Tokio documentation

## Phase 1: Setup

- [ ] T001 Create `src/agent/` directory structure with modules
- [ ] T002 Extract tool implementations into `src/agent/tools/`
- [ ] T003 Create `src/agent/context/` for session and history management
- [ ] T004 Create `src/agent/message.rs` with Message types
- [ ] T005 Configure `cargo.toml` workspace for new structure

## Quickstart

```bash
# Build the agent
cargo build

# Run version fast path
devil --version

# Start interactive session
devil run "analyze this codebase"

# Run single task
devil "add user authentication"
```
