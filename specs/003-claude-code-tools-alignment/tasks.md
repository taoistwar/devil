# Tasks: Claude Code Tools Alignment

**Input**: Design documents from `/specs/003-claude-code-tools-alignment/`
**Prerequisites**: plan.md, spec.md

**Tests**: Each tool requires 3 test cases (normal, boundary, error)

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Project Infrastructure)

**Purpose**: Shared infrastructure for Phase 2+ tools

### Task Module Infrastructure

- [ ] T001 Create task module directory `crates/agent/src/tools/task/` for task management tools
- [ ] T002 [P] Create task state types in `crates/agent/src/tools/task/types.rs` (TaskState, TaskStatus enum)
- [ ] T003 [P] Create task store using DashMap in `crates/agent/src/tools/task/store.rs`
- [ ] T004 [P] Create task scheduler using tokio::time in `crates/agent/src/tools/task/scheduler.rs`

### Planning Module Infrastructure

- [ ] T005 Create planning module directory `crates/agent/src/tools/planning/`
- [ ] T006 [P] Create AgentState enum with Planning variant in `crates/agent/src/tools/planning/state.rs`

### Worktree Module Infrastructure

- [ ] T007 Create worktree module directory `crates/agent/src/tools/worktree/`
- [ ] T008 [P] Create worktree state tracker in `crates/agent/src/tools/worktree/state.rs`

### MCP Module Infrastructure

- [ ] T009 Verify existing `crates/mcp` module integration points

**Checkpoint**: Infrastructure ready - tool implementation can begin

---

## Phase 2: User Story 1 - Core File & Shell Tools (Priority: P1)

**Goal**: Implement NotebookEdit, REPL, PowerShell tools for complete file/shell coverage

**Independent Test**: Run `devil run "使用 NotebookEdit 编辑 .ipynb 文件"` and verify

### Tests for User Story 1

- [ ] T010 [P] [US1] Test NotebookEdit normal case in `crates/agent/src/tools/file_tools/tests.rs`
- [ ] T011 [P] [US1] Test NotebookEdit error handling (invalid JSON)
- [ ] T012 [P] [US1] Test REPLTool normal case
- [ ] T013 [P] [US1] Test PowerShellTool on non-Windows (expected failure)

### Implementation for User Story 1

- [ ] T014 [US1] Implement NotebookEditTool input/output types in `crates/agent/src/tools/file_tools/notebook.rs`
- [ ] T015 [US1] Implement .ipynb JSON parsing and cell editing logic
- [ ] T016 [US1] Implement REPLTool in `crates/agent/src/tools/file_tools/repl.rs`
- [ ] T017 [US1] Implement PowerShellTool in `crates/agent/src/tools/file_tools/powershell.rs`
- [ ] T018 [US1] Register tools in `Agent::register_default_tools()` in `crates/agent/src/core.rs`

---

## Phase 3: User Story 4 - Planning & Worktree Tools (Priority: P1)

**Goal**: Implement EnterPlanMode, ExitPlanMode, EnterWorktree, ExitWorktree

**Independent Test**: Run `devil run "使用 EnterPlanMode 进入规划模式"` and verify

### Tests for User Story 4

- [ ] T019 [P] [US4] Test EnterPlanModeTool state transition
- [ ] T020 [P] [US4] Test ExitPlanModeTool state transition
- [ ] T021 [P] [US4] Test EnterWorktreeTool git worktree creation
- [ ] T022 [P] [US4] Test ExitWorktreeTool cleanup

### Implementation for User Story 4

- [ ] T023 [US4] Implement EnterPlanModeTool in `crates/agent/src/tools/planning/enter_plan_mode.rs`
- [ ] T024 [US4] Implement ExitPlanModeTool in `crates/agent/src/tools/planning/exit_plan_mode.rs`
- [ ] T025 [US4] Implement EnterWorktreeTool in `crates/agent/src/tools/worktree/enter.rs`
- [ ] T026 [US4] Implement ExitWorktreeTool in `crates/agent/src/tools/worktree/exit.rs`
- [ ] T027 [US4] Register tools in `Agent::register_default_tools()`

---

## Phase 4: User Story 5 - Task Management Tools (Priority: P1)

**Goal**: Implement TaskCreate, TaskUpdate, TaskList, TaskGet, TaskStop, TaskOutput

**Independent Test**: Run `devil run "使用 TaskCreate 创建任务，使用 TaskList 查看任务"` and verify

### Tests for User Story 5

- [ ] T028 [P] [US5] Test TaskCreateTool creates task with UUID
- [ ] T029 [P] [US5] Test TaskUpdateTool status transitions
- [ ] T030 [P] [US5] Test TaskListTool returns all tasks
- [ ] T031 [P] [US5] Test TaskGetTool returns specific task
- [ ] T032 [P] [US5] Test TaskStopTool terminates running task
- [ ] T033 [P] [US5] Test TaskOutputTool retrieves task output

### Implementation for User Story 5

- [ ] T034 [US5] Implement TaskCreateTool in `crates/agent/src/tools/task/task_create.rs`
- [ ] T035 [US5] Implement TaskUpdateTool in `crates/agent/src/tools/task/task_update.rs`
- [ ] T036 [US5] Implement TaskListTool in `crates/agent/src/tools/task/task_list.rs`
- [ ] T037 [US5] Implement TaskGetTool in `crates/agent/src/tools/task/task_get.rs`
- [ ] T038 [US5] Implement TaskStopTool in `crates/agent/src/tools/task/task_stop.rs`
- [ ] T039 [US5] Implement TaskOutputTool in `crates/agent/src/tools/task/task_output.rs`
- [ ] T040 [US5] Register tools in `Agent::register_default_tools()`

---

## Phase 5: User Story 6 - MCP Integration Tools (Priority: P1)

**Goal**: Implement MCPTool, ListMcpResources, ReadMcpResource, McpAuth

**Independent Test**: Run `devil run "使用 MCPTool 调用 MCP 服务器工具"` and verify

### Tests for User Story 6

- [ ] T041 [P] [US6] Test MCPTool JSON-RPC request/response
- [ ] T042 [P] [US6] Test ListMcpResourcesTool returns resource list
- [ ] T043 [P] [US6] Test ReadMcpResourceTool fetches resource content
- [ ] T044 [P] [US6] Test McpAuthTool authentication flow

### Implementation for User Story 6

- [ ] T045 [US6] Implement MCPTool in `crates/agent/src/tools/mcp/mcp_tool.rs`
- [ ] T046 [US6] Implement ListMcpResourcesTool in `crates/agent/src/tools/mcp/list_resources.rs`
- [ ] T047 [US6] Implement ReadMcpResourceTool in `crates/agent/src/tools/mcp/read_resource.rs`
- [ ] T048 [US6] Implement McpAuthTool in `crates/agent/src/tools/mcp/auth.rs`
- [ ] T049 [US6] Register tools in `Agent::register_default_tools()`

---

## Phase 6: User Story 7 - Configuration & Skills Tools (Priority: P2)

**Goal**: Implement ConfigTool, BriefTool, CtxInspectTool, SkillTool, DiscoverSkills

**Independent Test**: Run `devil run "使用 ConfigTool 查看配置，使用 SkillTool 列出技能"` and verify

### Tests for User Story 7

- [ ] T050 [P] [US7] Test ConfigTool get/set configuration
- [ ] T051 [P] [US7] Test BriefTool output mode switching
- [ ] T052 [P] [US7] Test CtxInspectTool context display
- [ ] T053 [P] [US7] Test SkillTool execution
- [ ] T054 [P] [US7] Test DiscoverSkillsTool listing

### Implementation for User Story 7

- [ ] T055 [US7] Implement ConfigTool in `crates/agent/src/tools/config/config_tool.rs`
- [ ] T056 [US7] Implement BriefTool in `crates/agent/src/tools/config/brief_tool.rs`
- [ ] T057 [US7] Implement CtxInspectTool in `crates/agent/src/tools/config/ctx_inspect.rs`
- [ ] T058 [US7] Implement SkillTool in `crates/agent/src/tools/skills/skill_tool.rs`
- [ ] T059 [US7] Implement DiscoverSkillsTool in `crates/agent/src/tools/skills/discover.rs`
- [ ] T060 [US7] Register tools in `Agent::register_default_tools()`

---

## Phase 7: User Story 8 - LSP Language Server Tools (Priority: P2)

**Goal**: Implement LSPTool for language server protocol

**Independent Test**: Run `devil run "使用 LSPTool 跳转到函数定义"` and verify

### Tests for User Story 8

- [ ] T061 [P] [US8] Test LSPTool initialize handshake
- [ ] T062 [P] [US8] Test LSPTool completion request/response
- [ ] T063 [P] [US8] Test LSPTool goto definition

### Implementation for User Story 8

- [ ] T064 [US8] Implement LSPTool in `crates/agent/src/tools/lsp/lsp_tool.rs`
- [ ] T065 [US8] Register tool in `Agent::register_default_tools()`

---

## Phase 8: User Story 9 - Scheduling & Workflow Tools (Priority: P2)

**Goal**: Implement CronCreate, CronDelete, CronList, WorkflowTool

**Independent Test**: Run `devil run "使用 CronCreate 创建定时任务，使用 CronList 查看"` and verify

### Tests for User Story 9

- [ ] T066 [P] [US9] Test CronCreateTool creates cron entry
- [ ] T067 [P] [US9] Test CronDeleteTool removes cron entry
- [ ] T068 [P] [US9] Test CronListTool lists all crons
- [ ] T069 [P] [US9] Test WorkflowTool executes workflow steps

### Implementation for User Story 9

- [ ] T070 [US9] Implement CronCreateTool in `crates/agent/src/tools/scheduling/cron_create.rs`
- [ ] T071 [US9] Implement CronDeleteTool in `crates/agent/src/tools/scheduling/cron_delete.rs`
- [ ] T072 [US9] Implement CronListTool in `crates/agent/src/tools/scheduling/cron_list.rs`
- [ ] T073 [US9] Implement WorkflowTool in `crates/agent/src/tools/workflow/workflow_tool.rs`
- [ ] T074 [US9] Register tools in `Agent::register_default_tools()`

---

## Phase 9: User Story 10 - Communication & Team Tools (Priority: P2)

**Goal**: Implement SendMessage, ListPeers, TeamCreate, TeamDelete

**Independent Test**: Run `devil run "使用 SendMessage 发送消息，使用 ListPeers 查看 peers"` and verify

### Tests for User Story 10

- [ ] T075 [P] [US10] Test SendMessageTool sends message
- [ ] T076 [P] [US10] Test ListPeersTool lists team members
- [ ] T077 [P] [US10] Test TeamCreateTool creates team
- [ ] T078 [P] [US10] Test TeamDeleteTool deletes team

### Implementation for User Story 10

- [ ] T079 [US10] Implement SendMessageTool in `crates/agent/src/tools/comm/send_message.rs`
- [ ] T080 [US10] Implement ListPeersTool in `crates/agent/src/tools/comm/list_peers.rs`
- [ ] T081 [US10] Implement TeamCreateTool in `crates/agent/src/tools/comm/team_create.rs`
- [ ] T082 [US10] Implement TeamDeleteTool in `crates/agent/src/tools/comm/team_delete.rs`
- [ ] T083 [US10] Register tools in `Agent::register_default_tools()`

---

## Phase 10: User Story 11 - Enhanced Tools (Priority: P3)

**Goal**: Implement AskUserQuestion, WebBrowser, Snip, SyntheticOutput, ReviewArtifact, SubscribePR, SuggestBackgroundPR, PushNotification, TerminalCapture, Monitor, Sleep, ToolSearch, RemoteTrigger

**Independent Test**: Run `devil run "使用 AskUserQuestion 询问用户"` and verify

### Tests for User Story 11

- [ ] T084 [P] [US11] Test AskUserQuestionTool prompts user
- [ ] T085 [P] [US11] Test WebBrowserTool renders page
- [ ] T086 [P] [US11] Test SnipTool captures screen
- [ ] T087 [P] [US11] Test SyntheticOutputTool generates output
- [ ] T088 [P] [US11] Test ReviewArtifactTool reviews content
- [ ] T089 [P] [US11] Test SubscribePRTool subscribes to PR
- [ ] T090 [P] [US11] Test SuggestBackgroundPRTool suggests background
- [ ] T091 [P] [US11] Test PushNotificationTool sends notification
- [ ] T092 [P] [US11] Test TerminalCaptureTool captures terminal
- [ ] T093 [P] [US11] Test MonitorTool monitors system
- [ ] T094 [P] [US11] Test SleepTool delays execution
- [ ] T095 [P] [US11] Test ToolSearchTool searches tools
- [ ] T096 [P] [US11] Test RemoteTriggerTool triggers remote

### Implementation for User Story 11

- [ ] T097 [US11] Implement AskUserQuestionTool in `crates/agent/src/tools/enhanced/ask_question.rs`
- [ ] T098 [US11] Implement WebBrowserTool in `crates/agent/src/tools/enhanced/web_browser.rs`
- [ ] T099 [US11] Implement SnipTool in `crates/agent/src/tools/enhanced/snip.rs`
- [ ] T100 [US11] Implement SyntheticOutputTool in `crates/agent/src/tools/enhanced/synthetic_output.rs`
- [ ] T101 [US11] Implement ReviewArtifactTool in `crates/agent/src/tools/enhanced/review_artifact.rs`
- [ ] T102 [US11] Implement SubscribePRTool in `crates/agent/src/tools/enhanced/subscribe_pr.rs`
- [ ] T103 [US11] Implement SuggestBackgroundPRTool in `crates/agent/src/tools/enhanced/suggest_background_pr.rs`
- [ ] T104 [US11] Implement PushNotificationTool in `crates/agent/src/tools/enhanced/push_notification.rs`
- [ ] T105 [US11] Implement TerminalCaptureTool in `crates/agent/src/tools/enhanced/terminal_capture.rs`
- [ ] T106 [US11] Implement MonitorTool in `crates/agent/src/tools/enhanced/monitor.rs`
- [ ] T107 [US11] Implement SleepTool in `crates/agent/src/tools/enhanced/sleep.rs`
- [ ] T108 [US11] Implement ToolSearchTool in `crates/agent/src/tools/enhanced/tool_search.rs`
- [ ] T109 [US11] Implement RemoteTriggerTool in `crates/agent/src/tools/enhanced/remote_trigger.rs`
- [ ] T110 [US11] Register all enhanced tools in `Agent::register_default_tools()`

---

## Phase 11: Enhanced Capabilities

**Purpose**: Implement enhancements beyond Claude Code baseline

### Enhanced Features

- [ ] T111 [P] Implement Bash command history in `crates/agent/src/tools/builtin.rs`
- [ ] T112 [P] Implement Read syntax highlighting markers
- [ ] T113 [P] Implement Glob exclude patterns
- [ ] T114 [P] Implement tool result streaming

---

## Phase 12: Polish & Cross-Cutting Concerns

**Purpose**: Final integration and validation

### Integration & Validation

- [ ] T115 [P] Run `cargo build` to verify all tools compile
- [ ] T116 [P] Run `cargo clippy` to verify code quality
- [ ] T117 [P] Update tools module exports in `crates/agent/src/tools.rs`
- [ ] T118 [P] Verify all 53 tools registered in `Agent::register_default_tools()`
- [ ] T119 Run integration test: execute each tool once
- [ ] T120 Update SPEC.md completion status

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: No dependencies - starts immediately
- **Phase 2-10 (User Stories)**: Depend on Phase 1 completion
- **Phase 11-12 (Polish)**: Depend on all user stories complete

### Within Each User Story

- Tests (T0xx) can run in parallel [P]
- Implementation tasks depend on infrastructure tasks
- Core implementation before integration

### Parallel Opportunities

All tasks marked [P] can execute in parallel within their phase.

---

## Summary

| Metric | Count |
|--------|-------|
| **Total Tasks** | 120 |
| **Setup Tasks** | 9 |
| **User Story Tasks** | 97 |
| **Polish Tasks** | 6 |
| **Parallelizable [P]** | ~100 |

### Task Count Per User Story

| User Story | Tasks | Tools |
|------------|-------|-------|
| US1 | 9 | NotebookEdit, REPL, PowerShell |
| US4 | 10 | EnterPlanMode, ExitPlanMode, EnterWorktree, ExitWorktree |
| US5 | 14 | TaskCreate, TaskUpdate, TaskList, TaskGet, TaskStop, TaskOutput |
| US6 | 11 | MCPTool, ListMcpResources, ReadMcpResource, McpAuth |
| US7 | 12 | ConfigTool, BriefTool, CtxInspectTool, SkillTool, DiscoverSkills |
| US8 | 5 | LSPTool |
| US9 | 9 | CronCreate, CronDelete, CronList, WorkflowTool |
| US10 | 10 | SendMessage, ListPeers, TeamCreate, TeamDelete |
| US11 | 27 | AskUserQuestion, WebBrowser, Snip, etc. |

### Suggested MVP Scope

**Phase 1 (Setup) + Phase 4 (User Story 5 - Task Management)** = MVP

This delivers the core task management system which other tools can build upon.
