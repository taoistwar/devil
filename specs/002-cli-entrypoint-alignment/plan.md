# Implementation Plan: CLI Entry Point Alignment

**Branch**: `260417-feat-cli-entrypoint` | **Date**: 2026-04-17 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification for aligning with Claude Code CLI entry points

## Summary

Re-implement the CLI entry point to align with Claude Code's `cli.tsx`, providing a structured command dispatcher with fast paths for common operations (version, help), proper initialization, and graceful shutdown handling.

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)
**Primary Dependencies**: `clap` (CLI argument parsing), `tokio` (async runtime), `anyhow`/`thiserror` (error handling)
**Storage**: Config files in `~/.devil/config.toml`, sessions in `~/.devil/sessions/`
**Testing**: Integration tests in `tests/cli_*.rs`
**Target Platform**: Linux/macOS terminal environments
**Project Type**: CLI tool / terminal application
**Performance Goals**: Version flag response under 100ms, graceful shutdown under 2 seconds
**Constraints**: Must maintain backward compatibility with existing `run` and `repl` commands
**Scale/Scope**: Single-user terminal sessions, no concurrent multi-user requirements

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

- [x] **I. Rust-First Standards**: Implementation uses idiomatic Rust with `cargo clippy` cleanliness
- [x] **II. Tokio Concurrency Model**: Async operations use Tokio runtime
- [x] **III. Claude Code Reference Parity**: CLI structure follows `cli.tsx` patterns
- [x] **IV. Robust Error Handling**: `anyhow`/`thiserror` for proper error propagation
- [x] **V. Tool-First Architecture**: CLI is the primary interface

## Project Structure

### Documentation (this feature)

```
specs/002-cli-entrypoint-alignment/
├── spec.md              # Feature specification
├── tasks.md             # Task list (this file output)
├── plan.md              # Implementation plan
└── checklists/          # Quality checklists
    └── requirements.md
```

### Source Code (repository root)

```
src/
├── main.rs              # Entry point with version fast-path
├── cli/                 # CLI module (NEW)
│   ├── mod.rs           # Module declarations
│   ├── dispatcher.rs    # Command dispatcher
│   ├── commands/        # Command implementations
│   │   ├── mod.rs
│   │   ├── version.rs
│   │   ├── help.rs
│   │   ├── run.rs
│   │   ├── repl.rs
│   │   └── config.rs
│   ├── init.rs          # Initialization system
│   └── error.rs         # CLI error types
├── config/              # Configuration module (enhanced)
│   ├── mod.rs
│   └── settings.rs
└── lib.rs               # Agent library

tests/
├── cli_version_test.rs
├── cli_help_test.rs
├── cli_run_test.rs
├── cli_repl_test.rs
├── cli_config_test.rs
└── cli_shutdown_test.rs
```

**Structure Decision**: CLI module added under `src/cli/` with command registry pattern

## Complexity Tracking

No complexity violations - straightforward CLI restructuring.

## Phase 0: Research (Completed)

- [x] Analyzed `references/claude-code/src/entrypoints/cli.tsx` - 339 lines, multiple fast paths
- [x] Analyzed `references/claude-code/src/entrypoints/init.ts` - 352 lines, initialization chain
- [x] Documented key patterns: version fast path, command dispatch, graceful shutdown

## Phase 1: Setup

- [ ] T001 Create `src/cli/mod.rs` with module declarations
- [ ] T002 Add `clap` dependency to `Cargo.toml`
- [ ] T003 Configure logging infrastructure

## Phase 2: Foundational

- [ ] T004-T011 Command dispatcher and individual commands
- [ ] T012-T014 Init system and signal handling
- [ ] T015-T018 Config loading with environment override

## Phase 3-9: User Stories

Each user story implements a specific CLI feature following the dispatcher pattern.

## Quickstart

```bash
# Build
cargo build

# Test version fast path
devil --version

# Test help
devil --help

# Run single task
devil run "echo hello"

# Enter REPL
devil repl
```
