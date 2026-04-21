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

- **Worker crash/unresponsive**: Worker failures detected within 10 seconds; coordinator retries via SendMessage with corrected instructions, or reassigns task.
- **Circular spawn requests**: Detected and prevented by `MAX_SUBAGENT_DEPTH` (3) limit - spawn requests beyond depth are rejected.
- **Resource exhaustion (max workers)**: When `MAX_CONCURRENT_WORKERS` (4) is reached, coordinator returns `ResourceExhausted` error (E3003) per SPEC_DEPENDENCIES.md.
- **Infinite recursion prevention**: `MAX_SUBAGENT_DEPTH` (3) enforced at runtime - spawn attempts beyond limit are denied.
- **MCP tools unavailable**: Worker reports error via `<task-notification status="failed">` with error details.
- **Worker timeout**: Worker exceeding `WORKER_TIMEOUT_MS` (5 min) is marked as `timeout` status and cleaned up.
- **Max tool calls exceeded**: Worker is gracefully terminated with partial results if `MAX_WORKER_TOOL_CALLS` (1000) is reached.

## Constants & Configuration

### Worker Agent Tool Set

```typescript
// ASYNC_AGENT_ALLOWED_TOOLS - Worker 允许使用的工具集
const ASYNC_AGENT_ALLOWED_TOOLS = [
  'FileRead',      // 文件读取
  'WebSearch',     // 网页搜索
  'TodoWrite',     // 任务管理
  'Grep',          // 内容搜索
  'WebFetch',      // 网页抓取
  'Glob',          // 文件匹配
  'Shell',         // Bash/PowerShell
  'FileEdit',      // 文件编辑
  'FileWrite',     // 文件写入
  'NotebookEdit',  // Jupyter笔记本编辑
  'Skill',         // 技能调用
  'SyntheticOutput', // 合成输出
  'ToolSearch',    // 工具搜索
  'EnterWorktree', // 进入worktree
  'ExitWorktree',  // 退出worktree
] as const;

// INTERNAL_WORKER_TOOLS - 仅协调者可用的内部编排工具
const INTERNAL_WORKER_TOOLS = [
  'TeamCreate',    // 创建Agent团队
  'TeamDelete',    // 删除Agent团队
  'SendMessage',   // 发送消息（协调者专用）
] as const;
```

### Resource Limits

```typescript
// 最大并发 Worker 数量
const MAX_CONCURRENT_WORKERS = 4;

// 最大子Agent层级深度
const MAX_SUBAGENT_DEPTH = 3;

// Worker 超时时间（毫秒）
const WORKER_TIMEOUT_MS = 300000; // 5 minutes

// 单个Worker最大工具调用次数
const MAX_WORKER_TOOL_CALLS = 1000;
```

### Agent-to-Agent Communication Protocol

Workers communicate with Coordinator via structured message passing:

```typescript
// Worker → Coordinator 消息格式
interface WorkerMessage {
  type: 'task-update' | 'task-notification' | 'error-report';
  task-id: string;
  timestamp: string;
  payload: WorkerUpdate | TaskNotification | ErrorReport;
}

// Coordinator → Worker 消息格式
interface CoordinatorMessage {
  type: 'task-assign' | 'task-cancel' | 'retry-instruction';
  task-id: string;
  timestamp: string;
  payload: TaskAssignment | TaskCancel | RetryInstruction;
}
```

### Result Aggregation Format

```xml
<!-- <task-notification> XML 格式定义 -->
<task-notification>
  <task-id>string</task-id>
  <status>completed|failed|killed|timeout</status>
  <summary>string</summary>
  <result>
    <!-- 任务实际输出内容，可包含多行 -->
    <content type="text|json|xml"/>
    <files>
      <file path="..." modified="true|false"/>
    </files>
  </result>
  <usage>
    <tools-invoked count="N">
      <tool name="..." calls="N" duration-ms="N"/>
    </tools-invoked>
    <tokens prompt="N" completion="N"/>
    <duration-ms>N</duration-ms>
  </usage>
  <parent-task-id>string</parent-task-id>
  <created-at>ISO8601 timestamp</created-at>
  <completed-at>ISO8601 timestamp</completed-at>
</task-notification>
```

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a mechanism to activate coordinator mode in which the main agent can spawn worker agents.
- **FR-002**: System MUST allow coordinator to spawn worker agents with explicitly restricted tool sets.
- **FR-003**: Worker agents MUST only have access to permitted tools defined in `ASYNC_AGENT_ALLOWED_TOOLS`. Internal orchestration tools (`TeamCreate`, `TeamDelete`, `SendMessage`) are blocked for workers.
- **FR-004**: Worker agents MUST be able to spawn sub-agents with their own tool restrictions.
- **FR-005**: Coordinator MUST receive structured results from worker agents upon task completion in `<task-notification>` XML format as defined in Result Aggregation Format section.
- **FR-006**: System MUST provide a way to track active workers and their assigned tasks.
- **FR-007**: Coordinator MUST aggregate results from multiple workers into coherent output.
- **FR-008**: System MUST handle worker failures with: (1) First retry via SendMessage to same worker with corrected instructions (worker has error context), (2) If retry fails, try different approach or report failure to user.
- **FR-009**: Tool restrictions MUST be enforced at runtime - workers MUST NOT bypass restrictions.
- **FR-010**: System MUST limit maximum concurrent workers to `MAX_CONCURRENT_WORKERS` (4).
- **FR-011**: System MUST limit maximum sub-agent hierarchy depth to `MAX_SUBAGENT_DEPTH` (3).
- **FR-012**: System MUST enforce resource limits including worker timeout (`WORKER_TIMEOUT_MS`) and max tool calls (`MAX_WORKER_TOOL_CALLS`).
- **FR-013**: Workers and Coordinator MUST communicate via the defined message protocol (WorkerMessage/CoordinatorMessage formats).

### Resource Constraints

- **RC-001**: Maximum concurrent worker agents MUST NOT exceed `MAX_CONCURRENT_WORKERS` (4).
- **RC-002**: Maximum sub-agent hierarchy depth MUST NOT exceed `MAX_SUBAGENT_DEPTH` (3).
- **RC-003**: Each worker MUST have a timeout of `WORKER_TIMEOUT_MS` (300000ms / 5 minutes).
- **RC-004**: Each worker MUST be limited to `MAX_WORKER_TOOL_CALLS` (1000) tool invocations per session.
- **RC-005**: When resource limits are reached, system MUST return `ResourceExhausted` error (E3003) per SPEC_DEPENDENCIES.md.

### Key Entities

- **Coordinator**: The main agent responsible for task decomposition, worker spawning, and result aggregation.
- **Worker**: A subordinate agent with restricted tools assigned to execute specific subtasks.
- **SubAgent**: An agent spawned by a worker, inheriting restrictions but potentially with narrower scope.
- **ToolSet**: A collection of permitted tools that defines what actions an agent can perform.
- **Task**: A unit of work assigned to a worker, containing description, expected output format, and deadline if applicable.
- **Result**: Structured output in `<task-notification>` XML format containing task-id, status (completed|failed|killed|timeout), summary, result text, and usage statistics.
- **WorkerMessage**: Structured message sent from Worker to Coordinator containing task updates, notifications, or error reports.
- **CoordinatorMessage**: Structured message sent from Coordinator to Worker containing task assignments, cancellations, or retry instructions.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Coordinator MUST NOT spawn more than `MAX_CONCURRENT_WORKERS` (4) concurrent workers - attempts beyond limit return `ResourceExhausted` error.
- **SC-002**: Worker tool restrictions are 100% enforced - restricted tools return denial on every attempt.
- **SC-003**: Results from all workers are aggregated within 5 seconds of the last worker completing.
- **SC-004**: Sub-agent spawning chain MUST respect `MAX_SUBAGENT_DEPTH` (3) limit - attempts to spawn beyond limit are rejected.
- **SC-005**: Worker failures are detected and reported to coordinator within 10 seconds.
- **SC-006**: System handles worker crash/restart without coordinator losing overall state.
- **SC-007**: Worker timeout is enforced at `WORKER_TIMEOUT_MS` (5 minutes) - timed-out workers are marked as `timeout` status.
- **SC-008**: All Worker-to-Coordinator communication follows the defined message protocol.

## Assumptions

- The existing agent framework already supports basic tool invocation and agent state management.
- MCP tools are available and can be dynamically enabled/disabled for worker agents.
- Workers communicate with coordinator via message passing with structured result formats following the defined communication protocol.
- Agent spawning is capped at `MAX_SUBAGENT_DEPTH` (3) levels to prevent infinite recursion.
- Resources (memory, CPU) are sufficient to support multiple concurrent agents within `MAX_CONCURRENT_WORKERS` (4) limit.
- The existing session management can track multiple active agents simultaneously.
- Resource limits are enforced at the coordinator level before spawning new workers.
