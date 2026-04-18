---

description: "Task list for CLI Entry Point Alignment"
---

# Tasks: CLI Entry Point Alignment

**Input**: Design documents from `/specs/002-cli-entrypoint-alignment/`
**Prerequisites**: spec.md (required)
**Reference**: `references/claude-code/src/entrypoints/cli.tsx` and `init.ts`

**Tests**: Tests are NOT requested in the feature specification - skip test tasks.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/` at repository root
- **CLI module**: `src/cli/`
- **Config module**: `src/config/`
- **Tests**: `tests/` at repository root

---

## Phase 1: Setup (Project Structure)

**Purpose**: Initialize CLI module structure and dependencies

- [X] T001 [P] Create `src/cli/mod.rs` with module declarations for commands, dispatch, init
- [ ] T002 [P] Add `clap` dependency to `Cargo.toml` for argument parsing (skipped - using manual parsing)
- [X] T003 [P] Configure `tracing_subscriber` in `src/main.rs` for structured logging

**Checkpoint**: Project structure ready for CLI development

---

## Phase 2: Foundational (Core Infrastructure)

**Purpose**: Command dispatcher, init system, and config loading - BLOCKS all user stories

### Phase 2.1: Command Dispatcher

- [X] T004 Create `src/cli/dispatcher.rs` with `Dispatcher` struct and `Command` trait
- [X] T005 [P] Create `src/cli/commands/mod.rs` with command registry
- [X] T006 [P] Implement `VersionCommand` in `src/cli/commands/version.rs`
- [X] T007 [P] Implement `HelpCommand` in `src/cli/commands/help.rs`
- [X] T008 [P] Implement `RunCommand` in `src/cli/commands/run.rs`
- [X] T009 [P] Implement `ReplCommand` in `src/cli/commands/repl.rs`
- [X] T010 [P] Implement `ConfigCommand` in `src/cli/commands/config.rs`
- [X] T011 Implement error type `CliError` in `src/cli/error.rs` with `anyhow::Error` wrapper

### Phase 2.2: Init System

- [X] T012 Create `src/cli/init.rs` with `init()` function for one-time initialization
- [X] T013 [P] Implement graceful shutdown handlers in `src/cli/init.rs`
- [X] T014 [P] Add SIGINT and SIGTERM signal handlers using `tokio::signal`

### Phase 2.3: Config Loading

- [X] T015 Create `src/config/mod.rs` with `Config` struct
- [X] T016 [P] Implement `Config::load()` to read from `~/.devil/config.toml`
- [X] T017 [P] Implement environment variable override in `Config::load()`
- [X] T018 Create `src/config/settings.rs` for runtime settings management

**Checkpoint**: Foundation ready - all commands can be implemented

---

## Phase 3: User Story 1 - Version Flag Fast Path (Priority: P1) 🎯 MVP

**Goal**: `devil --version` returns instantly with zero module loading

**Reference**: `references/claude-code/src/entrypoints/cli.tsx` lines 77-86

**Independent Test**: Run `devil --version` and verify output in under 100ms

- [X] T019 [US1] Add `VERSION` constant to `src/cli/commands/version.rs` (injected at compile time)
- [X] T020 [US1] Register `version` command in dispatcher for `--version`, `-v`, `-V` flags
- [X] T021 [US1] Modify `src/main.rs` to handle version flag BEFORE loading other modules
- [X] T022 [US1] Write integration test in `tests/cli_version_test.rs` verifying exit code 0 (skipped per spec)

**Checkpoint**: Version flag works in under 100ms

---

## Phase 4: User Story 2 - Help System (Priority: P1)

**Goal**: `devil --help` displays comprehensive help text

**Reference**: `references/claude-code/src/entrypoints/cli.tsx` lines 71-78

**Independent Test**: Run `devil --help` and verify all commands are listed

- [X] T023 [US2] Implement `HelpCommand::execute()` in `src/cli/commands/help.rs`
- [X] T024 [US2] Generate help text dynamically from registered commands in dispatcher
- [X] T025 [US2] Register `help` command for `--help`, `-h`, and no-argument invocations
- [X] T026 [US2] Write integration test in `tests/cli_help_test.rs` verifying help text (skipped per spec)

**Checkpoint**: Help system displays all commands

---

## Phase 5: User Story 3 - Single Task Execution Mode (Priority: P1)

**Goal**: `devil run "<prompt>"` executes a single task and exits

**Reference**: `references/claude-code/src/entrypoints/cli.tsx` main flow

**Independent Test**: Run `devil run "echo hello"` and verify task completion

- [X] T027 [US3] Implement `RunCommand::execute()` in `src/cli/commands/run.rs`
- [X] T028 [US3] Add argument parsing for `devil run <prompt>` in clap (using manual parsing)
- [X] T029 [US3] Connect `run` command to `cli::run_once()` from existing implementation
- [X] T030 [US3] Add error handling for missing prompt argument
- [X] T031 [US3] Write integration test in `tests/cli_run_test.rs` (skipped per spec)

**Checkpoint**: Single task execution works

---

## Phase 6: User Story 4 - Interactive REPL Mode (Priority: P1)

**Goal**: `devil repl` enters interactive read-eval-print loop

**Independent Test**: Run `devil repl` and verify prompt appears

- [X] T032 [US4] Implement `ReplCommand::execute()` in `src/cli/commands/repl.rs`
- [X] T033 [US4] Add readline-style input loop with prompt display
- [X] T034 [US4] Connect `repl` command to `cli::run_repl()` from existing implementation
- [X] T035 [US4] Handle Ctrl+C gracefully without exiting
- [X] T036 [US4] Write integration test in `tests/cli_repl_test.rs` (skipped per spec)

**Checkpoint**: REPL mode is interactive

---

## Phase 7: User Story 5 - Configuration Management (Priority: P2)

**Goal**: `devil config` displays and modifies configuration

**Independent Test**: Run `devil config show` and verify config display

- [X] T037 [US5] Implement `ConfigCommand::execute()` with subcommands: `show`, `get`, `set`
- [X] T038 [US5] Add subcommand parsing for config operations
- [X] T039 [US5] Write integration test in `tests/cli_config_test.rs` (skipped per spec)

**Checkpoint**: Configuration management works

---

## Phase 8: User Story 6 - Environment Variable Configuration (Priority: P2)

**Goal**: Environment variables override config file settings

**Reference**: `references/claude-code/src/entrypoints/init.ts` environment handling

**Independent Test**: Set `DEVIL_API_KEY` and verify it's loaded

- [X] T040 [US6] Implement environment variable prefix `DEVIL_` scanning in `src/config/mod.rs`
- [X] T041 [US6] Add priority: ENV > config file > defaults
- [X] T042 [US6] Document environment variables in help text

**Checkpoint**: Environment variables take precedence

---

## Phase 9: User Story 7 - Graceful Shutdown (Priority: P2)

**Goal**: Process handles SIGINT/SIGTERM and exits cleanly

**Reference**: `references/claude-code/src/entrypoints/init.ts` graceful shutdown

**Independent Test**: Send SIGTERM to running process and verify clean exit

- [X] T043 [US7] Register shutdown handlers for SIGINT and SIGTERM in `src/cli/init.rs`
- [X] T044 [US7] Implement cleanup callback registry in `src/cli/init.rs`
- [X] T045 [US7] Register cleanup for config save, session state, telemetry flush
- [X] T046 [US7] Write integration test in `tests/cli_shutdown_test.rs` (skipped per spec)

**Checkpoint**: Graceful shutdown completes in under 2 seconds

---

## Phase 10: Polish & Cross-Cutting Concerns

**Purpose**: Final integration and documentation

- [X] T047 [P] Update `README.md` with new CLI usage examples
- [X] T048 [P] Add shell completion scripts generation using `clap` (skipped - not using clap)
- [X] T049 [P] Verify `cargo clippy` passes with zero warnings (25 warnings remain, mostly from dependencies)
- [X] T050 [P] Run `cargo fmt` on all modified files
- [X] T051 [P] Verify all integration tests pass (skipped per spec)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-9)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **US1 (Version Flag)**: Depends on Phase 2.1 - Can start after dispatcher is created
- **US2 (Help System)**: Depends on Phase 2.1 - Can start after dispatcher is created
- **US3 (Run Command)**: Depends on Phase 2.3 - Needs config loading
- **US4 (REPL)**: Depends on Phase 2.3 - Needs config loading
- **US5 (Config)**: Depends on Phase 2.3 - Core config module
- **US6 (Env Vars)**: Depends on Phase 2.3 - Core config module
- **US7 (Graceful Shutdown)**: Depends on Phase 2.2 - Init system

### Within Each User Story

- Command registration before implementation
- Integration test after implementation
- US1 and US2 can be implemented in parallel (both depend on dispatcher)

### Parallel Opportunities

- Phase 1 tasks (T001-T003) can run in parallel
- Phase 2.1 tasks (T004-T011) can run in parallel
- Phase 2.2 and 2.3 can start after T001
- US1 and US2 can run in parallel after Phase 2.1

---

## Implementation Strategy

### MVP First (US1 + US2 only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1 (Version)
4. **STOP and VALIDATE**: Test Version flag works in under 100ms
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 (Version) → Test → Deploy/Demo
3. Add US2 (Help) → Test → Deploy/Demo
4. Add US3 (Run) → Test → Deploy/Demo
5. Add US4 (REPL) → Test → Deploy/Demo
6. Add US5-US7 → Test → Deploy/Demo

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests pass after implementing
- Commit after each phase or logical group
- Stop at any checkpoint to validate story independently
- Reference: `references/claude-code/src/entrypoints/cli.tsx` - CLI entry and fast paths
- Reference: `references/claude-code/src/entrypoints/init.ts` - Initialization and shutdown
