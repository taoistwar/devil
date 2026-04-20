# Tasks: Multi-Agent Coordinator

**Input**: Design documents from `/specs/008-multi-agent-coordinator/`
**Prerequisites**: plan.md (required), spec.md (required for user stories)

**Tests**: Tests are NOT explicitly requested in spec - skip test generation

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Structure)

**Purpose**: Verify existing coordinator/subagent module structure is ready for enhancement

- [ ] T001 Verify existing module exports in `crates/agent/src/lib.rs` include `coordinator` and `subagent`
- [ ] T002 [P] Review `coordinator/mod.rs` module documentation completeness
- [ ] T003 [P] Review `subagent/mod.rs` module documentation completeness

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that enables all user stories - existing types are foundation

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Review `coordinator/types.rs` - verify `TaskNotification`, `WorkerDirective`, `CoordinatorConfig` are complete
- [ ] T005 Review `subagent/types.rs` - verify `SubagentType`, `SubagentParams` are complete
- [ ] T006 Verify `mode_detection.rs` - `is_coordinator_mode()`, `enable_coordinator_mode()`, `match_session_mode()` implementation
- [ ] T007 Verify `worker_agent.rs` - `get_worker_system_prompt()`, `is_worker_tool_available()` implementation
- [ ] T008 Verify `recursion_guard.rs` - max depth enforcement mechanism

**Checkpoint**: Foundation types verified - user story implementation can now begin

---

## Phase 3: User Story 1 - Coordinator Mode Activation (Priority: P1) 🎯 MVP

**Goal**: System can activate coordinator mode and display active worker count

**Independent Test**: Trigger coordinator mode and verify status shows active workers

### Implementation for User Story 1

- [X] T009 [US1] Implement `get_coordinator_status()` in `crates/agent/src/coordinator/orchestration.rs`
- [X] T010 [US1] Add status display format in `crates/agent/src/coordinator/orchestration.rs` showing active worker count
- [X] T011 [US1] Add `CoordinatorStatus` response type in `crates/agent/src/coordinator/types.rs`
- [X] T012 [US1] Create `/coordinator` CLI command in `crates/agent/src/commands/coordinator.rs` with status subcommand
- [X] T013 [US1] Wire `coordinator/mod.rs` exports for new status functionality

**Checkpoint**: Coordinator mode can be activated and status queried ✅

---

## Phase 4: User Story 2 - Worker Agent Spawning (Priority: P1)

**Goal**: Coordinator can spawn workers with restricted tool sets

**Independent Test**: Spawn a worker and verify it only has permitted tools

### Implementation for User Story 2

- [X] T014 [P] [US2] Resolve TODOs in `orchestration.rs:continue_task()` - implement actual SendMessage flow
- [X] T015 [P] [US2] Implement `spawn_worker()` method in `crates/agent/src/coordinator/orchestration.rs` that creates SubagentParams with worker tools
- [X] T016 [US2] Integrate `SubagentExecutor` into orchestrator worker spawn flow in `crates/agent/src/coordinator/orchestration.rs`
- [X] T017 [US2] Add tool restriction verification in `worker_agent.rs` - verify restricted tools return denial
- [X] T018 [US2] Add Worker spawn notification format in `crates/agent/src/coordinator/types.rs`

**Checkpoint**: Workers can be spawned with enforced tool restrictions ✅

---

## Phase 5: User Story 3 - Hierarchical Sub-Agent Spawning (Priority: P2)

**Goal**: Workers can spawn sub-agents with their own tool restrictions

**Independent Test**: Worker spawns sub-agent and verifies depth tracking works

### Implementation for User Story 3

- [ ] T019 [P] [US3] Review `context_inheritance.rs` for worker-to-subagent context flow
- [ ] T020 [P] [US3] Review `recursion_guard.rs` for max depth enforcement (should already be complete)
- [ ] T021 [US3] Add worker sub-agent spawn capability in `worker_agent.rs` using `SubagentExecutor`
- [ ] T022 [US3] Add depth tracking to `RunningTask` in `orchestration.rs`
- [ ] T023 [US3] Implement depth limit check before sub-agent spawn

**Checkpoint**: Sub-agents can be spawned from workers with depth enforcement

---

## Phase 6: User Story 4 - Work Distribution and Result Aggregation (Priority: P1)

**Goal**: Coordinator aggregates results from multiple workers

**Independent Test**: Multiple workers complete and coordinator aggregates their outputs

### Implementation for User Story 4

- [X] T024 [P] [US4] Enhance `synthesize_research()` in `orchestration.rs` to handle multiple TaskNotifications
- [X] T025 [P] [US4] Implement `aggregate_results()` method in `orchestration.rs`
- [X] T026 [US4] Add failure detection and `on_task_failed()` handler in `orchestration.rs`
- [X] T027 [US4] Implement task reassignment logic in `orchestration.rs` when worker fails
- [X] T028 [US4] Add timeout handling for worker tasks

**Checkpoint**: Results are aggregated and failures are handled gracefully ✅

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T029 [P] Update `crates/agent/src/coordinator/mod.rs` module documentation with new methods
- [ ] T030 [P] Update `crates/agent/src/subagent/mod.rs` module documentation
- [ ] T031 Update `quickstart.md` with new API examples
- [ ] T032 Run `cargo clippy --package devil-agent` and address any warnings
- [ ] T033 Run `cargo test --package devil-agent` and ensure all pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1, US2, US4 are P1 - can proceed in parallel after Foundational
  - US3 is P2 - can proceed in parallel with P1 stories
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **US1 (P1)**: Can start after Foundational - No dependencies on other stories
- **US2 (P1)**: Can start after Foundational - May integrate with US1 but independently testable
- **US3 (P2)**: Can start after Foundational - May integrate with US1/US2 but independently testable
- **US4 (P1)**: Can start after Foundational - Aggregates results from US2

### Within Each User Story

- Core types before integration
- Basic implementation before enhancement
- Story complete before moving to next priority

---

## Parallel Opportunities

- T002 and T003 can run in parallel (different modules)
- T014 and T015 can run in parallel (different methods)
- T019 and T020 can run in parallel (different files)
- T024 and T025 can run in parallel (different methods)
- T029 and T030 can run in parallel (different modules)

---

## Implementation Strategy

### MVP First (US1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: US1
4. **STOP and VALIDATE**: Test coordinator mode activation independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add US1 → Test independently → Deploy/Demo (MVP!)
3. Add US2 + US4 → Test independently → Deploy/Demo
4. Add US3 → Test independently → Deploy/Demo
5. Polish → Final deploy

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Existing `coordinator/` and `subagent/` modules provide strong foundation - most work is enhancement
