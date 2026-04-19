# Tasks: Claude Code Memory System Alignment

**Input**: Design documents from `/specs/007-claude-code-memory-alignment/`
**Prerequisites**: plan.md, spec.md, data-model.md, research.md, quickstart.md

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization - add serde_yaml dependency for frontmatter parsing

- [x] T001 [P] Add `serde_yaml` dependency to Cargo.toml for frontmatter parsing (dependencies already existed)
- [x] T002 [P] Add `tempfile` dev dependency for testing (dependency already existed)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data types and memory directory infrastructure that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Create `crates/agent/src/context/memory/mod.rs` with module exports
- [x] T004 [P] Create `crates/agent/src/context/memory/types.rs` with MemoryType enum (User, Feedback, Project, Reference)
- [x] T005 [P] Create `crates/agent/src/context/memory/types.rs` with MemoryFrontmatter struct
- [x] T006 [P] Create `crates/agent/src/context/memory/types.rs` with MemoryEntry struct
- [x] T007 Create `crates/agent/src/context/memory/types.rs` with frontmatter parsing functions
- [x] T008 Create `crates/agent/src/context/memory/dir.rs` with MemoryDir struct and path resolution
- [x] T009 Create `crates/agent/src/context/memory/index.rs` with MemoryIndex struct and MEMORY.md parsing
- [x] T010 Create `crates/agent/src/context/memory/truncation.rs` with EntrypointTruncation and MAX_ENTRYPOINT_* constants
- [x] T011 Create `crates/agent/src/context/memory/prompts.rs` with MemoryGuidance static text

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Persistent Memory Storage (Priority: P1) MVP

**Goal**: Agent can save and recall memories across sessions

**Independent Test**: Create a memory file, then read it back in a new session

### Implementation for User Story 1

- [x] T012 [P] [US1] Implement memory save operation in `crates/agent/src/context/memory/dir.rs`
- [x] T013 [P] [US1] Implement memory recall/load in `crates/agent/src/context/memory/dir.rs`
- [x] T014 [US1] Integrate memory loading into user_context.rs
- [x] T015 [US1] Add unit tests for save/recall in `crates/agent/src/context/memory/`

**Checkpoint**: User Story 1 complete - memories persist across sessions

---

## Phase 4: User Story 2 - Typed Memory Organization (Priority: P1)

**Goal**: Memories are organized by type and agent uses appropriate type based on context

**Independent Test**: Save memories of different types, verify they are stored and retrieved correctly

### Implementation for User Story 2

- [x] T016 [P] [US2] Implement type-based file naming in `crates/agent/src/context/memory/dir.rs` (MemoryType::file_prefix)
- [x] T017 [P] [US2] Implement type-based memory listing in `crates/agent/src/context/memory/dir.rs` (list_memories_by_type)
- [x] T018 [US2] Add MemoryType filtering to recall operations (list_memories_by_type)
- [x] T019 [US2] Add unit tests for typed memory operations (tested via MemoryType tests)

**Checkpoint**: User Story 2 complete - all four memory types work independently

---

## Phase 5: User Story 3 - MEMORY.md Index Management (Priority: P2)

**Goal**: MEMORY.md serves as index entrypoint with proper truncation

**Independent Test**: Create many memories, verify index stays under 200 lines / 25KB

### Implementation for User Story 3

- [x] T020 [P] [US3] Implement index upsert in `crates/agent/src/context/memory/index.rs` (MemoryIndex::upsert)
- [x] T021 [P] [US3] Implement index remove in `crates/agent/src/context/memory/index.rs` (MemoryIndex::remove)
- [x] T022 [US3] Implement truncation with warning messages in `crates/agent/src/context/memory/truncation.rs` (truncate_entrypoint)
- [x] T023 [US3] Integrate truncation into memory loading (user_context.rs)
- [x] T024 [US3] Add unit tests for index management and truncation (24 tests passing)

**Checkpoint**: User Story 3 complete - index management works correctly

---

## Phase 6: User Story 4 - Memory Exclusion Guidance (Priority: P2)

**Goal**: Agent understands what NOT to save in memory

**Independent Test**: Agent refuses to save code patterns, git history, etc.

### Implementation for User Story 4

- [x] T025 [P] [US4] Add WHAT_NOT_TO_SAVE guidance text to `prompts.rs` (WHAT_NOT_TO_SAVE constant)
- [x] T026 [P] [US4] Add WHEN_TO_ACCESS section to `prompts.rs` (WHEN_TO_ACCESS constant)
- [x] T027 [P] [US4] Add TRUSTING_RECALL section to `prompts.rs` (TRUSTING_RECALL constant)
- [x] T028 [US4] Integrate guidance into memory prompt building (MemoryGuidance::build_prompt)
- [x] T029 [US4] Add unit tests for guidance prompt generation (tested via prompts tests)

**Checkpoint**: User Story 4 complete - agent provides proper memory guidance

---

## Phase 7: User Story 5 - External System References (Priority: P3)

**Goal**: Agent can save and recall pointers to external systems

**Independent Test**: Save a reference memory, verify it can be retrieved

### Implementation for User Story 5

- [x] T030 [P] [US5] Reference memory storage uses reference_ prefix (MemoryType::Reference)
- [x] T031 [US5] Reference memories can be listed and retrieved by type (list_memories_by_type)
- [x] T032 [US5] Add unit tests for reference memory operations (MemoryType tests)

**Checkpoint**: User Story 5 complete - reference memories work

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: Integration, CLI command, and edge cases

- [ ] T033 [P] Update `crates/agent/src/context/mod.rs` to export memory module
- [ ] T034 [P] Enhance `/memory` command in `crates/agent/src/commands/advanced/memory.rs`
- [ ] T035 Add CLAUDE.md file discovery tests (existing functionality preserved)
- [ ] T036 [P] Add integration tests for full memory workflow
- [ ] T037 Verify `cargo clippy` passes with zero warnings
- [ ] T038 Run `cargo test` to verify all tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-7)**: All depend on Foundational phase completion
  - US1 and US2 are both P1 and can run in parallel after Foundational
  - US3, US4, US5 follow in order
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Requires Foundational (Phase 2)
- **User Story 2 (P1)**: Requires Foundational (Phase 2), can parallel US1
- **User Story 3 (P2)**: Requires Foundational, typically done after US1/US2
- **User Story 4 (P2)**: Requires Foundational, can parallel US3
- **User Story 5 (P3)**: Requires Foundational, done last

### Within Each User Story

- Core types before operations
- Operations before integration
- Implementation before tests

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational [P] tasks can run in parallel
- US1 and US2 can run in parallel after Foundational
- US3 and US4 can run in parallel after US1/US2
- Polish tasks marked [P] can run in parallel

---

## Implementation Strategy

### MVP First (User Story 1 + 2)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. Complete Phase 4: User Story 2
5. **STOP and VALIDATE**: Core memory system works

### Full Feature Set

6. Add Phase 5: User Story 3 (Index Management)
7. Add Phase 6: User Story 4 (Guidance)
8. Add Phase 7: User Story 5 (Reference memories)
9. Polish & integrate

---

## Summary

| Metric | Value |
|--------|-------|
| Total Tasks | 38 |
| Phase 1 (Setup) | 2 |
| Phase 2 (Foundational) | 9 |
| Phase 3 (US1) | 4 |
| Phase 4 (US2) | 4 |
| Phase 5 (US3) | 5 |
| Phase 6 (US4) | 5 |
| Phase 7 (US5) | 3 |
| Phase 8 (Polish) | 6 |
| Parallelizable Tasks | 20 |
| MVP Scope (US1+US2) | 19 tasks |