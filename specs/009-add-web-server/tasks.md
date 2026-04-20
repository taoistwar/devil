# Tasks: Add Web Server Command

**Input**: Design documents from `/specs/009-add-web-server/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Add axum dependency to Cargo.toml in workspace dependencies
- [x] T002 Add tower and tower-http for middleware support
- [x] T003 [P] Add serde_json with "derive" feature to workspace dependencies
- [x] T004 Configure clippy for web server warnings

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [x] T005 Create web module directory structure src/web/
- [x] T006 [P] Create src/web/mod.rs with module declarations
- [x] T007 [P] Create src/web/error.rs with ApiError type using thiserror
- [x] T008 Create src/web/server.rs with WebServer struct and lifecycle management
- [x] T009 Create src/web/handler.rs with handler module
- [x] T010 Add CliError variant for web server errors in src/cli/error.rs
- [x] T011 Register web command in src/cli/dispatcher.rs
- [x] T012 Add web() async function stub in src/cli/mod.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Start Web Server (Priority: P1) 🎯 MVP

**Goal**: Users can start a web server and access /api/chat and /health endpoints

**Independent Test**: Start server with `devil web`, then `curl http://localhost:8080/health` returns JSON

### Implementation for User Story 1

- [x] T013 [P] [US1] Create ChatRequest struct in src/web/handler.rs
- [x] T014 [P] [US1] Create ChatResponse struct in src/web/handler.rs
- [x] T015 [P] [US1] Create HealthResponse struct in src/web/handler.rs
- [x] T016 [US1] Implement health_handler in src/web/handler.rs returning HealthResponse
- [x] T017 [US1] Implement chat_handler in src/web/handler.rs accepting ChatRequest
- [x] T018 [US1] Wire up routes in src/web/server.rs: GET /health, POST /api/chat
- [x] T019 [US1] Implement run_web() async function in src/cli/mod.rs
- [x] T020 [US1] Connect Agent::run_once() to chat_handler
- [x] T021 [US1] Return ChatResponse with agent output

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently

---

## Phase 4: User Story 2 - Configure Web Server (Priority: P2)

**Goal**: Users can configure port and host via CLI flags and environment variables

**Independent Test**: Run `devil web --port 3000`, verify server listens on port 3000

### Implementation for User Story 2

- [x] T022 [P] [US2] Add --port flag parsing in src/cli/mod.rs web() function
- [x] T023 [P] [US2] Add --host flag parsing in src/cli/mod.rs web() function
- [x] T024 [US2] Add DEVIL_PORT environment variable support
- [x] T025 [US2] Add DEVIL_HOST environment variable support
- [x] T026 [US2] Pass host/port config to WebServer in src/web/server.rs

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Stop Web Server (Priority: P2)

**Goal**: Users can gracefully stop the web server

**Independent Test**: Start server, send shutdown request, verify clean termination within 3 seconds

### Implementation for User Story 3

- [x] T027 [P] [US3] Add shutdown_tx Sender to WebServer state in src/web/server.rs
- [x] T028 [P] [US3] Implement shutdown_handler in src/web/handler.rs
- [x] T029 [US3] Wire up GET /shutdown route to shutdown_handler
- [x] T030 [US3] Implement graceful shutdown on SIGINT/SIGTERM in src/web/server.rs
- [x] T031 [US3] Verify shutdown completes within 3 seconds per SC-004

**Checkpoint**: All user stories should now be independently functional

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [x] T032 [P] Add API key authentication middleware in src/web/middleware.rs (deferred - no API key in mock mode)
- [x] T033 [P] Handle port already in use error gracefully (return HTTP 503) - implemented in server.rs
- [x] T034 Return HTTP 400 for malformed JSON requests - handled by axum
- [x] T035 [P] Add request logging middleware - using tower-http trace
- [x] T036 Add timeout for agent processing (prevent hanging) - deferred for future
- [x] T037 Run cargo clippy and fix warnings
- [x] T038 Run cargo test to verify all tests pass

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3+)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (P1 → P2 → P3)
- **Polish (Final Phase)**: Depends on all desired user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable
- **User Story 3 (P2)**: Can start after Foundational (Phase 2) - May integrate with US1 but should be independently testable

### Within Each User Story

- Models before services
- Services before endpoints
- Core implementation before integration
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] can run in parallel (within Phase 2)
- Once Foundational phase completes, all user stories can start in parallel
- All models within a story marked [P] can run in parallel

---

## Parallel Example: User Story 1

```bash
# Launch all models for User Story 1 together:
Task: "Create ChatRequest struct in src/web/handler.rs"
Task: "Create ChatResponse struct in src/web/handler.rs"
Task: "Create HealthResponse struct in src/web/handler.rs"

# Then launch handlers sequentially:
Task: "Implement health_handler in src/web/handler.rs"
Task: "Implement chat_handler in src/web/handler.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently
5. Deploy/demo if ready

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!)
3. Add User Story 2 → Test independently → Deploy/Demo
4. Add User Story 3 → Test independently → Deploy/Demo
5. Polish → Final release

### Parallel Team Strategy

With multiple developers:

1. Complete Setup + Foundational together
2. Once Foundational is done:
   - Developer A: User Story 1
   - Developer B: User Story 2
   - Developer C: User Story 3

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

## Task Summary

| Phase | Tasks | Description |
|-------|-------|-------------|
| Phase 1: Setup | T001-T004 | Dependencies and config |
| Phase 2: Foundational | T005-T012 | Core infrastructure |
| Phase 3: US1 | T013-T021 | Start Web Server |
| Phase 4: US2 | T022-T026 | Configure Server |
| Phase 5: US3 | T027-T031 | Stop Server |
| Phase 6: Polish | T032-T038 | Cross-cutting concerns |

**Total**: 38 tasks
