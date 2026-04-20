# Feature Specification: Multi-Agent Coordinator

**Feature Branch**: `260419-feat-multi-agent-coordinator`  
**Created**: 2026-04-19  
**Status**: Draft  
**Input**: User description: "在 claude code 中 references/claude-code 下，coordinator/ — 多 Agent 协调特性。位于 coordinatorMode.ts 实现了多 Agent 协调模式： 主 Agent 作为"协调者"分配工作 Worker Agent 通过 AgentTool 生成 Worker 拥有受限的工具集（Bash、Edit、Read、MCP 工具） Worker 可以进一步生成子 Agent 在我当前项目中，对齐此功能"

## Clarifications

### Session 2026-04-19

- Q: Worker 工具集具体包含哪些工具？ → A: FileRead, WebSearch, TodoWrite, Grep, WebFetch, Glob, Shell (Bash/PowerShell), FileEdit, FileWrite, NotebookEdit, Skill, SyntheticOutput, ToolSearch, EnterWorktree, ExitWorktree（参考 `constants/tools.ts:ASYNC_AGENT_ALLOWED_TOOLS`）
- Q: "Structured results" 具体是什么格式？ → A: `<task-notification>` XML 格式，包含 task-id, status, summary, result, usage 元素（参考 `coordinatorMode.ts:146-160`）
- Q: "Gracefully" 处理 worker 失败具体是什么行为？ → A: 失败时先用 SendMessage 继续同一 worker（因为有完整错误上下文），如果重试失败则换方案或报告用户（参考 `coordinatorMode.ts:231-233`）
- Q: Worker 的 SendMessage 是否属于受限工具？ → A: 是的，SendMessage 是内部编排工具，仅协调者可用（参考 `constants/tools.ts:INTERNAL_WORKER_TOOLS`）
- Q: Sub-agent 结果如何回传？ → A: 通过 worker 作为中介回传，worker 收到子 agent 结果后整合到自己的结果中（参考 `coordinatorMode.ts:173-174`）

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Coordinator Mode Activation (Priority: P1)

A user initiates a complex task that requires coordination of multiple specialized agents. The system automatically or on request enables coordinator mode, where the main agent orchestrates work distribution.

**Why this priority**: This is the foundational capability - without coordinator mode activation, no multi-agent coordination can occur.

**Independent Test**: Can be tested by triggering coordinator mode and verifying the main agent enters coordinator state ready to spawn workers.

**Acceptance Scenarios**:

1. **Given** a running agent session, **When** user requests a complex task or triggers coordinator mode, **Then** the main agent enters coordinator state and can spawn worker agents.
2. **Given** coordinator mode is active, **When** user queries status, **Then** system displays active worker count and available coordination commands.

---

### User Story 2 - Worker Agent Spawning (Priority: P1)

The coordinator generates worker agents with restricted tool sets to handle specific subtasks in parallel.

**Why this priority**: Core functionality - workers are the execution units that actually perform distributed work.

**Independent Test**: Can be tested by instructing coordinator to spawn a worker for a specific task and verifying the worker operates with only permitted tools.

**Acceptance Scenarios**:

1. **Given** coordinator mode is active, **When** coordinator assigns a task to spawn a worker, **Then** a new worker agent is created with only FileRead, WebSearch, TodoWrite, Grep, WebFetch, Glob, Shell, FileEdit, FileWrite, NotebookEdit, Skill, SyntheticOutput, ToolSearch, EnterWorktree, ExitWorktree available (TeamCreate, TeamDelete, SendMessage blocked).
2. **Given** a worker agent is spawned, **When** worker attempts to use a tool not in its permitted set, **Then** the tool execution is denied with an appropriate message.
3. **Given** a worker agent completes its task, **When** worker reports results to coordinator, **Then** coordinator receives structured output from the worker.

---

### User Story 3 - Hierarchical Sub-Agent Spawning (Priority: P2)

Worker agents can further spawn their own sub-agents to handle nested complexity, forming an agent hierarchy.

**Why this priority**: Enables handling of deeply nested task structures - a worker may encounter a subtask that itself requires parallelization.

**Independent Test**: Can be tested by instructing a worker to spawn a sub-agent and verifying the sub-agent operates with appropriate restrictions.

**Acceptance Scenarios**:

1. **Given** a worker agent is active, **When** worker determines a subtask requires parallelization, **Then** worker can spawn a sub-agent with appropriate tool restrictions.
2. **Given** a sub-agent is spawned by a worker, **When** sub-agent completes, **Then** results flow back through the worker to the coordinator.

---

### User Story 4 - Work Distribution and Result Aggregation (Priority: P1)

The coordinator distributes tasks to workers and aggregates their results into coherent outputs.

**Why this priority**: Without result aggregation, distributed work has no value - the coordinator must synthesize worker outputs.

**Independent Test**: Can be tested by assigning multiple parallel tasks to workers and verifying coordinator aggregates all results correctly.

**Acceptance Scenarios**:

1. **Given** multiple workers are active with assigned tasks, **When** all workers complete, **Then** coordinator aggregates results into a unified response.
2. **Given** a worker fails or times out, **When** coordinator detects failure, **Then** coordinator can reassign the task or report failure appropriately.

---

### Edge Cases

- What happens when a worker agent crashes or becomes unresponsive?
- How does the system handle circular spawn requests (worker spawning worker spawning original)?
- What happens when coordinator runs out of resources to spawn new workers?
- How does the system prevent infinite recursion in sub-agent spawning?
- What happens when MCP tools used by workers become unavailable?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a mechanism to activate coordinator mode in which the main agent can spawn worker agents.
- **FR-002**: System MUST allow coordinator to spawn worker agents with explicitly restricted tool sets.
- **FR-003**: Worker agents MUST only have access to permitted tools (FileRead, WebSearch, TodoWrite, Grep, WebFetch, Glob, Shell, FileEdit, FileWrite, NotebookEdit, Skill, SyntheticOutput, ToolSearch, EnterWorktree, ExitWorktree). Internal orchestration tools (TeamCreate, TeamDelete, SendMessage) are blocked for workers.
- **FR-004**: Worker agents MUST be able to spawn sub-agents with their own tool restrictions.
- **FR-005**: Coordinator MUST receive structured results from worker agents upon task completion in `<task-notification>` XML format: `<task-id>`, `<status>`, `<summary>`, `<result>`, `<usage>` elements.
- **FR-006**: System MUST provide a way to track active workers and their assigned tasks.
- **FR-007**: Coordinator MUST aggregate results from multiple workers into coherent output.
- **FR-008**: System MUST handle worker failures with: (1) First retry via SendMessage to same worker with corrected instructions (worker has error context), (2) If retry fails, try different approach or report failure to user.
- **FR-009**: Tool restrictions MUST be enforced at runtime - workers MUST NOT bypass restrictions.
- **FR-010**: System SHOULD limit maximum nesting depth for hierarchical agent spawning.

### Key Entities

- **Coordinator**: The main agent responsible for task decomposition, worker spawning, and result aggregation.
- **Worker**: A subordinate agent with restricted tools assigned to execute specific subtasks.
- **SubAgent**: An agent spawned by a worker, inheriting restrictions but potentially with narrower scope.
- **ToolSet**: A collection of permitted tools that defines what actions an agent can perform.
- **Task**: A unit of work assigned to a worker, containing description, expected output format, and deadline if applicable.
- **Result**: Structured output in `<task-notification>` XML format containing task-id, status (completed|failed|killed), summary, result text, and usage statistics.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Coordinator can spawn at least 4 concurrent worker agents without degradation.
- **SC-002**: Worker tool restrictions are 100% enforced - restricted tools return denial on every attempt.
- **SC-003**: Results from all workers are aggregated within 5 seconds of the last worker completing.
- **SC-004**: Sub-agent spawning chain supports at least 3 levels of depth.
- **SC-005**: Worker failures are detected and reported to coordinator within 10 seconds.
- **SC-006**: System handles worker crash/restart without coordinator losing overall state.

## Assumptions

- The existing agent framework already supports basic tool invocation and agent state management.
- MCP tools are available and can be dynamically enabled/disabled for worker agents.
- Workers communicate with coordinator via message passing with structured result formats.
- Agent spawning is not infinitely recursive - the reference implementation does not enforce a maximum depth, but practical limits apply based on resource availability.
- Resources (memory, CPU) are sufficient to support multiple concurrent agents.
- The existing session management can track multiple active agents simultaneously.
