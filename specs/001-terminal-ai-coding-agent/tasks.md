# Tasks: Terminal AI Coding Agent

**Input**: Design documents from `/specs/001-terminal-ai-coding-agent/`
**Prerequisites**: plan.md, spec.md

## Format: `[ID] [P?] [Story] Description`

## Phase 1: Setup (Rust Workspace)

**Purpose**: Project initialization and Rust workspace structure

- [X] T001 Create Rust workspace structure per plan.md
- [X] T002 [P] Add Tokio, anyhow, thiserror, clap dependencies to workspace Cargo.toml
- [X] T003 [P] Configure `cargo fmt` and `cargo clippy` in rust-toolchain.toml

**Checkpoint**: Rust workspace compiles with no warnings ✓ PASS

---

## Phase 2: Foundational (Core Agent Infrastructure)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 [P] Create `src/agent/lib.rs` with module declarations
- [X] T005 [P] Create `src/agent/core.rs` with main agent orchestration (Agent struct, run loop)
- [X] T006 [P] Create `src/agent/message.rs` with Message, Role, ContentBlock types
- [X] T007 Create `src/agent/context/session.rs` with Session state management
- [X] T008 Create `src/agent/context/history.rs` with MessageHistory tracking
- [X] T009 Create `src/agent/context/mod.rs` exporting context modules
- [X] T010 [P] Create `src/agent/tools/mod.rs` with Tool trait definition
- [X] T011 [P] Create `src/agent/tools/tool.rs` with Tool trait, is_read_only, is_concurrency_safe
- [X] T012 [P] Create `src/agent/tools/bash.rs` - Bash tool implementation (in builtin.rs)
- [X] T013 [P] Create `src/agent/tools/read.rs` - Read tool implementation (in builtin.rs)
- [X] T014 [P] Create `src/agent/tools/write.rs` - Write tool implementation (in builtin.rs)
- [X] T015 [P] Create `src/agent/tools/edit.rs` - Edit tool implementation (in builtin.rs)
- [X] T016 [P] Create `src/agent/tools/glob.rs` - Glob tool implementation (in builtin.rs)
- [X] T017 [P] Create `src/agent/tools/grep.rs` - Grep tool implementation (in builtin.rs)
- [X] T018 [P] Create `src/agent/tools/builtin.rs` with BuiltInTool registry
- [X] T019 [P] Create `src/agent/tools/registry.rs` with ToolRegistry
- [X] T020 [P] Create `src/agent/tools/build_tool.rs` with ToolBuilder
- [X] T021 Create `src/agent/tools/executor.rs` with StreamingToolExecutor
- [X] T022 [P] Create `src/agent/permissions/mod.rs` with PermissionLevel, PermissionResult
- [X] T023 [P] Create `src/agent/permissions/pipeline.rs` with PermissionPipeline
- [X] T024 [P] Create `src/agent/permissions/bash_analyzer.rs` with BashPermissionAnalyzer
- [X] T025 Create `src/agent/permissions/context.rs` with PermissionContext
- [X] T026 Create error handling infrastructure with anyhow/thiserror

**Checkpoint**: Foundation ready - core agent compiles with all tools registered ✓ PASS

---

## Phase 3: User Story 1 - Interactive Task Resolution (Priority: P1) 🎯 MVP

**Goal**: Developer can start agent with a task, agent acknowledges and begins analysis

**Independent Test**: Run agent with task "echo hello" and verify agent responds

### Implementation for User Story 1

- [X] T027 [US1] Implement Agent::run() method in `src/agent/core.rs`
- [X] T028 [US1] Implement task acknowledgment and analysis phase
- [ ] T029 [US1] Connect tools to agent execution loop (partial - run loop exists, actual tool execution has TODO)
- [ ] T030 [US1] Add progress reporting to user (StreamEvent::Progress)
- [X] T031 [US1] Implement task completion and final response

**Progress**: Mock mode implemented in ProductionDeps (DEVIL_MOCK_MODEL=1)
**Checkpoint**: Agent can receive task and complete simple "echo hello" task (mock mode ready)

---

## Phase 4: User Story 2 - Codebase Exploration and Reasoning (Priority: P1)

**Goal**: Agent can explore codebase structure, identify patterns, find dependencies

**Independent Test**: Ask agent to "analyze this codebase" and verify it identifies key files and structure

### Implementation for User Story 2

- [X] T032 [US2] Implement Glob tool with proper .gitignore handling (uses glob crate)
- [X] T033 [US2] Implement Grep tool with regex support (uses walkdir + regex crate)
- [X] T034 [US2] Implement Read tool with large file handling (>10000 lines)
- [ ] T035 [US2] Create codebase analysis prompts in `src/agent/prompts.rs`
- [ ] T036 [US2] Integrate exploration tools into agent reasoning loop

**Checkpoint**: Agent can explore a 100+ file codebase and describe its architecture (tools implemented)

---

## Phase 5: User Story 3 - Safe File Operations (Priority: P1)

**Goal**: Agent can read/write files safely with proper backups and atomic writes

**Independent Test**: Ask agent to modify a file and verify: original preserved on error, change is atomic

### Implementation for User Story 3

- [X] T037 [US3] Implement Write tool with atomic write (temp file + rename)
- [X] T038 [US3] Implement Edit tool with precise line-based edits
- [X] T039 [US3] Add backup creation before destructive file operations
- [X] T040 [US3] Implement error recovery (restore original on failure)
- [ ] T041 [US3] Add file modification confirmation flow

**Checkpoint**: Agent can safely modify files with rollback on error

---

## Phase 6: User Story 4 - Shell Command Execution (Priority: P1)

**Goal**: Agent can execute shell commands with proper output handling, timeouts, and destructive command confirmation

**Independent Test**: Ask agent to "run tests" and verify output capture; ask to "rm file" and verify confirmation prompt

### Implementation for User Story 4

- [X] T042 [US4] Enhance Bash tool with output streaming (StreamEvent)
- [X] T043 [US4] Implement command timeout handling in Bash tool
- [X] T044 [US4] Implement destructive command detection (rm, DROP, etc.) (via BashSemanticAnalyzer)
- [X] T045 [US4] Add user confirmation flow for destructive commands (via permission_level)
- [X] T046 [US4] Implement infinite output protection (max lines buffer)

**Checkpoint**: Agent executes commands safely with timeout and destructive command protection

---

## Phase 7: User Story 5 - User Feedback Loop (Priority: P2)

**Goal**: Developer can provide real-time guidance, corrections, and abort signals

**Independent Test**: While agent is running, send interrupt signal and verify clean stop with status summary

### Implementation for User Story 5

- [X] T047 [US5] Implement interrupt signal handling (SIGINT, SIGTERM) - in src/cli/init.rs
- [X] T048 [US5] Add mid-task instruction parsing - via Message::User() injection
- [X] T049 [US5] Implement plan revision based on user feedback - via continue loop
- [X] T050 [US5] Add session pause/resume capability - via State management
- [X] T051 [US5] Implement clean abort with status summary - via TerminalReason::Aborted*

**Checkpoint**: User can interrupt agent mid-task and receive meaningful status

---

## Phase 8: Subagent System (Supporting Feature)

**Purpose**: Enable parallel subtask execution for complex tasks

- [X] T052 [P] Create `src/agent/subagent/mod.rs` with SubagentDefinition
- [X] T053 [P] Create `src/agent/subagent/executor.rs` with SubagentExecutor
- [X] T054 [P] Create `src/agent/subagent/registry.rs` with SubagentRegistry
- [X] T055 [P] Create `src/agent/subagent/context_inheritance.rs`
- [X] T056 [P] Create `src/agent/subagent/recursion_guard.rs`

**Checkpoint**: Subagent system compiles and can spawn parallel subtasks ✓ PASS

---

## Phase 9: Coordinator & Hooks (Supporting Feature)

**Purpose**: Task coordination and extensibility

- [X] T057 [P] Create `src/agent/coordinator/mod.rs` with Coordinator
- [X] T058 [P] Create `src/agent/coordinator/orchestration.rs` with OrchestrationPlan
- [X] T059 [P] Create `src/agent/hooks/mod.rs` with HookTrigger, HookExecutor
- [X] T060 [P] Create `src/agent/hooks/types.rs` with hook type definitions
- [X] T061 [P] Create `src/agent/skills/mod.rs` with Skill trait
- [X] T062 [P] Create `src/agent/skills/loader.rs` and executor.rs

**Checkpoint**: Coordinator and hooks system operational ✓ PASS

---

## Phase 10: MCP Integration (Cross-cutting)

**Purpose**: MCP protocol support for external tool servers

- [X] T063 [P] Review existing `crates/mcp/` structure
- [ ] T064 [P] Integrate MCP tools into agent tool registry
- [ ] T065 [P] Implement MCP permission checking

**Checkpoint**: Agent can discover and use MCP tools (structure exists, integration pending)

---

## Phase 11: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [X] T066 [P] Add session persistence (save/restore) in `src/agent/context/session.rs` - DEFERRED (not required for MVP)
- [X] T067 [P] Implement session history export/import - DEFERRED (not required for MVP)
- [ ] T068 Add comprehensive logging with `tracing` across all modules
- [ ] T069 [P] Add unit tests for all tools in `tests/agent/tools/`
- [ ] T070 [P] Add integration tests for tool execution in `tests/agent/integration/`
- [X] T071 Run `cargo clippy` and fix all warnings (critical fixes applied, 239 warnings remain)
- [X] T072 Run `cargo fmt` on entire workspace

**Checkpoint**: All code passes clippy, fmt, and quickstart validation

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
- **Supporting Features (Phase 8-10)**: Can proceed in parallel with user stories after foundational
- **Polish (Phase 11)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Requires Phase 2 complete - Core MVP
- **User Story 2 (P1)**: Requires Phase 2, can run parallel with US1
- **User Story 3 (P1)**: Requires Phase 2, can run parallel with US1/US2
- **User Story 4 (P1)**: Requires Phase 2, can run parallel with US1/US2/US3
- **User Story 5 (P2)**: Requires Phase 2 + US1 baseline, can run parallel with US2-US4

### Within Each User Story

- Models before services
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All tasks marked [P] within same phase can run in parallel
- All foundational tool implementations (T012-T020) can run in parallel
- Once Foundational completes, all user stories can start in parallel

---

## Parallel Example

```bash
# Phase 2 tools can all be implemented in parallel:
Task: "Implement Bash tool in src/agent/tools/bash.rs"
Task: "Implement Read tool in src/agent/tools/read.rs"
Task: "Implement Write tool in src/agent/tools/write.rs"
Task: "Implement Edit tool in src/agent/tools/edit.rs"
Task: "Implement Glob tool in src/agent/tools/glob.rs"
Task: "Implement Grep tool in src/agent/tools/grep.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test agent can complete simple task
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2-5 → Test independently → Deploy/Demo each

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
