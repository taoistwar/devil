# Feature Specification: Claude Code Tools Alignment

**Feature Branch**: `260418-feat-claude-code-tools-alignment`
**Created**: 2026-04-18
**Last Updated**: 2026-04-21
**Status**: In Progress
**Input**: "对齐 references/claude-code 的内置 tools，它有的当前项目也要有，工具名称要一样，让后对齐每一个工具，Claude code工具的功能当前项目也要有，而且还要进一步的完善。"

## Overview

本规范涵盖 Claude Code 全部 52 个内置工具的对齐实现。当前项目已实现 10 个核心工具，剩余 42 个工具需要按优先级分阶段实现。所有工具使用 Rust 实现，遵循 Tokio 并发模型和 Claude Code 五要素协议。

## Tool Taxonomy (工具分类体系)

### 分类总览

```
Tools (52)
├── Core (11)          - 核心文件/Shell/Web/Agent 工具
├── Task (6)           - 任务生命周期管理
├── MCP (4)            - Model Context Protocol 集成
├── Config (5)         - 配置与技能系统
├── LSP (1)            - Language Server Protocol
├── Schedule (4)       - 定时任务与工作流
├── Communication (5)  - 消息与团队协作
├── Enhanced (11)      - 增强型用户交互工具
└── Planning (5)       - 规划模式与 Git Worktree
```

### 详细分类

| Category | ID | Tools | Count |
|----------|----|-------|-------|
| Core | C | Bash, Glob, Grep, Read, Edit, Write, NotebookEdit, WebFetch, WebSearch, TodoWrite, Agent | 11 |
| Task | T | TaskCreate, TaskUpdate, TaskList, TaskGet, TaskStop, TaskOutput | 6 |
| MCP | M | MCPTool, ListMcpResources, ReadMcpResource, McpAuth | 4 |
| Config | G | ConfigTool, SkillTool, DiscoverSkills, Brief, CtxInspect | 5 |
| LSP | L | LSPTool | 1 |
| Schedule | S | CronCreate, CronDelete, CronList, WorkflowTool | 4 |
| Communication | P | SendMessage, ListPeers, TeamCreate, TeamDelete, RemoteTrigger | 5 |
| Enhanced | E | AskUserQuestion, WebBrowser, SnipTool, SyntheticOutput, ReviewArtifact, SubscribePR, SuggestBackgroundPR, PushNotification, TerminalCapture, Monitor, SleepTool, ToolSearch | 12 |
| Planning | W | EnterPlanMode, ExitPlanMode, EnterWorktree, ExitWorktree | 4 |

**总计**: 52 tools (Note: 用户提及 53 tools，实际清点为 52)

## Tool Dependencies (工具依赖关系)

### 依赖关系图

```
Bash (C01) ──┬── Read (C04) ──┬── Edit (C05)
             │                └── Write (C06)
             │
             ├── Glob (C03) ──┬── Grep (C02)
             │
             └── Agent (C11) ──┬── TaskCreate (T01)
                              ├── TaskUpdate (T02)
                              ├── TaskList (T03)
                              ├── TaskGet (T04)
                              ├── TaskStop (T05)
                              └── TaskOutput (T06)

WebFetch (C08) ──┬── WebSearch (C09)
                 └── WebBrowser (E02)

MCPTool (M01) ──┬── ListMcpResources (M02)
                ├── ReadMcpResource (M03)
                └── McpAuth (M04)

SkillTool (G02) ─── DiscoverSkills (G03)

EnterPlanMode (W01) ─── ExitPlanMode (W02)
EnterWorktree (W03) ─── ExitWorktree (W04)

ConfigTool (G01) ─┬── Brief (G04)
                  └── CtxInspect (G05)

SendMessage (P01) ──┬── ListPeers (P02)
                    ├── TeamCreate (P03)
                    ├── TeamDelete (P04)
                    └── RemoteTrigger (P05)

WorkflowTool (S04) ─── CronCreate (S01)
                       ├── CronDelete (S02)
                       └── CronList (S03)
```

### 依赖类型定义

| Dependency Type | Symbol | Description | Example |
|-----------------|--------|-------------|---------|
| Sequential | `A → B` | A 必须先于 B 执行 | `EnterPlanMode → ExitPlanMode` |
| Requires | `A ⇒ B` | A 的输出作为 B 的输入 | `Bash → Read` |
| Optional | `A -? B` | B 可选依赖 A | `ConfigTool -? Brief` |
| Conflict | `A ! B` | A 和 B 不能同时执行 | `EnterPlanMode ! ExitPlanMode` (并发) |

### 工具依赖矩阵

| Tool | ID | Dependencies | Depended By | Type |
|------|----|--------------|-------------|------|
| Bash | C01 | - | Read, Glob, Grep, Edit, Write, Agent | Base |
| Read | C04 | Bash | Edit, Write, Grep | Sequential |
| Edit | C05 | Read, Bash | - | Requires |
| Write | C06 | Read, Bash | - | Requires |
| Glob | C03 | Bash | Grep | Sequential |
| Grep | C02 | Bash, Glob | - | Sequential |
| NotebookEdit | C07 | Read, Write | - | Requires |
| WebFetch | C08 | - | WebBrowser | Sequential |
| WebSearch | C09 | - | - | - |
| TodoWrite | C10 | - | - | - |
| Agent | C11 | Bash | Task* | Sequential |
| TaskCreate | T01 | Agent | - | Sequential |
| TaskUpdate | T02 | TaskCreate | - | Requires |
| TaskList | T03 | - | - | - |
| TaskGet | T04 | TaskCreate | - | Requires |
| TaskStop | T05 | TaskCreate, TaskGet | - | Requires |
| TaskOutput | T06 | TaskCreate | - | Requires |
| MCPTool | M01 | - | ListMcpResources, ReadMcpResource, McpAuth | Sequential |
| ListMcpResources | M02 | MCPTool | - | Requires |
| ReadMcpResource | M03 | MCPTool | - | Requires |
| McpAuth | M04 | MCPTool | - | Requires |
| ConfigTool | G01 | - | Brief, CtxInspect | Sequential |
| SkillTool | G02 | - | DiscoverSkills | Sequential |
| DiscoverSkills | G03 | SkillTool | - | Requires |
| Brief | G04 | ConfigTool | - | Optional |
| CtxInspect | G05 | ConfigTool | - | Optional |
| LSPTool | L01 | - | - | - |
| CronCreate | S01 | - | CronList, WorkflowTool | Sequential |
| CronDelete | S02 | CronList | - | Requires |
| CronList | S03 | - | - | - |
| WorkflowTool | S04 | CronCreate | - | Sequential |
| SendMessage | P01 | - | ListPeers, TeamCreate, TeamDelete, RemoteTrigger | Sequential |
| ListPeers | P02 | SendMessage | - | Requires |
| TeamCreate | P03 | SendMessage | - | Requires |
| TeamDelete | P04 | SendMessage | - | Requires |
| RemoteTrigger | P05 | SendMessage | - | Requires |
| AskUserQuestion | E01 | - | - | - |
| WebBrowser | E02 | WebFetch | - | Requires |
| SnipTool | E03 | - | - | - |
| SyntheticOutput | E04 | - | - | - |
| ReviewArtifact | E05 | - | - | - |
| SubscribePR | E06 | - | - | - |
| SuggestBackgroundPR | E07 | - | - | - |
| PushNotification | E08 | - | - | - |
| TerminalCapture | E09 | Bash | - | Sequential |
| Monitor | E10 | - | - | - |
| SleepTool | E11 | - | - | - |
| ToolSearch | E12 | - | - | - |
| EnterPlanMode | W01 | - | ExitPlanMode | Sequential |
| ExitPlanMode | W02 | EnterPlanMode | - | Requires |
| EnterWorktree | W03 | Bash | ExitWorktree | Sequential |
| ExitWorktree | W04 | EnterWorktree | - | Requires |

## Implementation Priority (实现优先级)

### Phase 1 (P1 - Core Foundation) - 共 17 tools

| Priority | ID | Tool | Category | Dependencies | Estimated LOC |
|----------|----|------|----------|--------------|---------------|
| P1 | C01 | Bash | Core | - | 300 |
| P1 | C02 | Grep | Core | Bash | 200 |
| P1 | C03 | Glob | Core | Bash | 150 |
| P1 | C04 | Read | Core | Bash | 100 |
| P1 | C05 | Edit | Core | Read, Bash | 200 |
| P1 | C06 | Write | Core | Read, Bash | 150 |
| P1 | C07 | NotebookEdit | Core | Read, Write | 250 |
| P1 | C08 | WebFetch | Core | - | 200 |
| P1 | C09 | WebSearch | Core | - | 200 |
| P1 | C10 | TodoWrite | Core | - | 150 |
| P1 | C11 | Agent | Core | Bash | 500 |
| P1 | T01 | TaskCreate | Task | Agent | 100 |
| P1 | T02 | TaskUpdate | Task | TaskCreate | 80 |
| P1 | T03 | TaskList | Task | - | 80 |
| P1 | T04 | TaskGet | Task | TaskCreate | 80 |
| P1 | T05 | TaskStop | Task | TaskCreate, TaskGet | 100 |
| P1 | T06 | TaskOutput | Task | TaskCreate | 100 |

**Phase 1 完成标准**: 11 Core tools + 6 Task tools = 17 tools

### Phase 2 (P2 - System Integration) - 共 13 tools

| Priority | ID | Tool | Category | Dependencies | Estimated LOC |
|----------|----|------|----------|--------------|---------------|
| P2 | W01 | EnterPlanMode | Planning | - | 100 |
| P2 | W02 | ExitPlanMode | Planning | EnterPlanMode | 80 |
| P2 | W03 | EnterWorktree | Planning | Bash | 150 |
| P2 | W04 | ExitWorktree | Planning | EnterWorktree | 100 |
| P2 | M01 | MCPTool | MCP | - | 300 |
| P2 | M02 | ListMcpResources | MCP | MCPTool | 100 |
| P2 | M03 | ReadMcpResource | MCP | MCPTool | 100 |
| P2 | M04 | McpAuth | MCP | MCPTool | 150 |
| P2 | G01 | ConfigTool | Config | - | 100 |
| P2 | G02 | SkillTool | Config | - | 200 |
| P2 | G03 | DiscoverSkills | Config | SkillTool | 150 |
| P2 | G04 | Brief | Config | ConfigTool | 50 |
| P2 | G05 | CtxInspect | Config | ConfigTool | 80 |

**Phase 2 完成标准**: 4 Planning tools + 4 MCP tools + 5 Config tools = 13 tools

### Phase 3 (P3 - Enhanced Features) - 共 22 tools

| Priority | ID | Tool | Category | Dependencies | Estimated LOC |
|----------|----|------|----------|--------------|---------------|
| P3 | L01 | LSPTool | LSP | - | 400 |
| P3 | S01 | CronCreate | Schedule | - | 150 |
| P3 | S02 | CronDelete | Schedule | CronList | 80 |
| P3 | S03 | CronList | Schedule | - | 80 |
| P3 | S04 | WorkflowTool | Schedule | CronCreate | 200 |
| P3 | P01 | SendMessage | Communication | - | 150 |
| P3 | P02 | ListPeers | Communication | SendMessage | 80 |
| P3 | P03 | TeamCreate | Communication | SendMessage | 100 |
| P3 | P04 | TeamDelete | Communication | SendMessage | 100 |
| P3 | P05 | RemoteTrigger | Communication | SendMessage | 150 |
| P3 | E01 | AskUserQuestion | Enhanced | - | 100 |
| P3 | E02 | WebBrowser | Enhanced | WebFetch | 300 |
| P3 | E03 | SnipTool | Enhanced | - | 150 |
| P3 | E04 | SyntheticOutput | Enhanced | - | 100 |
| P3 | E05 | ReviewArtifact | Enhanced | - | 150 |
| P3 | E06 | SubscribePR | Enhanced | - | 100 |
| P3 | E07 | SuggestBackgroundPR | Enhanced | - | 100 |
| P3 | E08 | PushNotification | Enhanced | - | 80 |
| P3 | E09 | TerminalCapture | Enhanced | Bash | 100 |
| P3 | E10 | Monitor | Enhanced | - | 120 |
| P3 | E11 | SleepTool | Enhanced | - | 50 |
| P3 | E12 | ToolSearch | Enhanced | - | 100 |

**Phase 3 完成标准**: 1 LSP + 4 Schedule + 5 Communication + 12 Enhanced = 22 tools

## Tool Version Management (工具版本管理)

### 版本规范

每个工具遵循语义化版本 `major.minor.patch`:

- **major**: 不兼容的 API 变更
- **minor**: 向后兼容的功能添加
- **patch**: 向后兼容的 bug 修复

### 版本状态

| Tool | ID | Version | Status | Last Updated |
|------|----|---------|--------|--------------|
| Bash | C01 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Glob | C03 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Grep | C02 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Read | C04 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Edit | C05 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Write | C06 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| WebFetch | C08 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| WebSearch | C09 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| TodoWrite | C10 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| Agent | C11 | 1.0.0 | ✅ Implemented | 2026-04-18 |
| NotebookEdit | C07 | 0.0.0 | 🔄 Planned | - |
| TaskCreate | T01 | 0.0.0 | 🔄 Planned | - |
| TaskUpdate | T02 | 0.0.0 | 🔄 Planned | - |
| TaskList | T03 | 0.0.0 | 🔄 Planned | - |
| TaskGet | T04 | 0.0.0 | 🔄 Planned | - |
| TaskStop | T05 | 0.0.0 | 🔄 Planned | - |
| TaskOutput | T06 | 0.0.0 | 🔄 Planned | - |
| MCPTool | M01 | 0.0.0 | 🔄 Planned | - |
| ListMcpResources | M02 | 0.0.0 | 🔄 Planned | - |
| ReadMcpResource | M03 | 0.0.0 | 🔄 Planned | - |
| McpAuth | M04 | 0.0.0 | 🔄 Planned | - |
| ConfigTool | G01 | 0.0.0 | 🔄 Planned | - |
| SkillTool | G02 | 0.0.0 | 🔄 Planned | - |
| DiscoverSkills | G03 | 0.0.0 | 🔄 Planned | - |
| Brief | G04 | 0.0.0 | 🔄 Planned | - |
| CtxInspect | G05 | 0.0.0 | 🔄 Planned | - |
| LSPTool | L01 | 0.0.0 | 🔄 Planned | - |
| CronCreate | S01 | 0.0.0 | 🔄 Planned | - |
| CronDelete | S02 | 0.0.0 | 🔄 Planned | - |
| CronList | S03 | 0.0.0 | 🔄 Planned | - |
| WorkflowTool | S04 | 0.0.0 | 🔄 Planned | - |
| SendMessage | P01 | 0.0.0 | 🔄 Planned | - |
| ListPeers | P02 | 0.0.0 | 🔄 Planned | - |
| TeamCreate | P03 | 0.0.0 | 🔄 Planned | - |
| TeamDelete | P04 | 0.0.0 | 🔄 Planned | - |
| RemoteTrigger | P05 | 0.0.0 | 🔄 Planned | - |
| AskUserQuestion | E01 | 0.0.0 | 🔄 Planned | - |
| WebBrowser | E02 | 0.0.0 | 🔄 Planned | - |
| SnipTool | E03 | 0.0.0 | 🔄 Planned | - |
| SyntheticOutput | E04 | 0.0.0 | 🔄 Planned | - |
| ReviewArtifact | E05 | 0.0.0 | 🔄 Planned | - |
| SubscribePR | E06 | 0.0.0 | 🔄 Planned | - |
| SuggestBackgroundPR | E07 | 0.0.0 | 🔄 Planned | - |
| PushNotification | E08 | 0.0.0 | 🔄 Planned | - |
| TerminalCapture | E09 | 0.0.0 | 🔄 Planned | - |
| Monitor | E10 | 0.0.0 | 🔄 Planned | - |
| SleepTool | E11 | 0.0.0 | 🔄 Planned | - |
| ToolSearch | E12 | 0.0.0 | 🔄 Planned | - |
| EnterPlanMode | W01 | 0.0.0 | 🔄 Planned | - |
| ExitPlanMode | W02 | 0.0.0 | 🔄 Planned | - |
| EnterWorktree | W03 | 0.0.0 | 🔄 Planned | - |
| ExitWorktree | W04 | 0.0.0 | 🔄 Planned | - |

### 版本迁移策略

| From | To | Migration Guide |
|------|----|-----------------|
| 0.x.y → 1.0.0 | 首次稳定发布，需完整测试 |
| 1.0.0 → 1.1.0 | 添加可选参数，旧调用兼容 |
| 1.0.0 → 2.0.0 | 破坏性变更，需迁移文档 |

## Uniform Interface (统一接口规范)

### Tool Trait Definition

所有工具必须实现以下 trait:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub trait Tool: Send + Sync {
    const NAME: &'static str;
    const DESCRIPTION: &'static str;
    const VERSION: &'static str = "0.0.0";
    
    fn input_schema() -> Schema;
    fn output_schema() -> Schema;
    
    async fn execute(
        &self,
        input: Value,
        context: &ToolContext,
    ) -> Result<ToolResult, ToolError>;
    
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }
}
```

### Tool Context

```rust
#[derive(Debug, Clone)]
pub struct ToolContext {
    pub session_id: String,
    pub working_directory: PathBuf,
    pub environment: HashMap<String, String>,
    pub permissions: PermissionScope,
    pub timeout: Duration,
    pub cancellation_token: CancellationToken,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            working_directory: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            environment: std::env::vars().collect(),
            permissions: PermissionScope::default(),
            timeout: Duration::from_secs(300),
            cancellation_token: CancellationToken::new(),
        }
    }
}
```

### Tool Result

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<ToolError>,
    pub metadata: ToolMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub tool_name: String,
    pub tool_version: String,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
}
```

### Error Types (per SPEC_DEPENDENCIES.md)

```rust
pub enum ToolError {
    UserInput {
        code: ErrorCode,
        message: String,
        field: Option<String>,
    },
    Permission {
        code: ErrorCode,
        message: String,
        required_permission: String,
    },
    Resource {
        code: ErrorCode,
        message: String,
        resource_type: String,
    },
    ExternalService {
        code: ErrorCode,
        message: String,
        service: String,
    },
    Internal {
        code: ErrorCode,
        message: String,
        stack_trace: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    InvalidInput = 1001,
    MissingRequiredField = 1002,
    InvalidFormat = 1003,
    PermissionDenied = 2001,
    InsufficientPermissions = 2002,
    OperationNotAllowed = 2003,
    ResourceNotFound = 3001,
    ResourceConflict = 3002,
    ResourceExhausted = 3003,
    LLMProviderError = 4001,
    MCPConnectionError = 4002,
    NetworkError = 4003,
    InternalError = 5001,
    NotImplemented = 5002,
    UnexpectedState = 5003,
}
```

## User Scenarios & Testing

### User Story 1 - Core File & Shell Tools (Phase 1)

**Acceptance Scenarios**:

1. **Given** Agent needs to read file, **When** using `Read` tool, **Then** returns correct content
2. **Given** Agent needs to edit file, **When** using `Edit` tool, **Then** performs string replacement
3. **Given** Agent needs to write file, **When** using `Write` tool, **Then** creates or overwrites file
4. **Given** Agent needs to find files, **When** using `Glob` tool, **Then** returns matching file list
5. **Given** Agent needs to search code, **When** using `Grep` tool, **Then** returns matching lines and positions
6. **Given** Agent needs to execute shell command, **When** using `Bash` tool, **Then** executes and returns output
7. **Given** Agent needs to edit Jupyter notebook, **When** using `NotebookEdit` tool, **Then** parses and modifies .ipynb
8. **Given** Agent needs to fetch webpage, **When** using `WebFetch` tool, **Then** returns HTML content
9. **Given** Agent needs to search web, **When** using `WebSearch` tool, **Then** returns search results
10. **Given** Agent needs to manage task list, **When** using `TodoWrite` tool, **Then** creates and updates tasks
11. **Given** Agent needs to spawn sub-agent, **When** using `Agent` tool, **Then** creates and executes sub-agent

### User Story 2 - Task Management Tools (Phase 1)

**Acceptance Scenarios**:

1. **Given** Agent needs to create task, **When** using `TaskCreate` tool, **Then** creates task and returns task_id
2. **Given** Agent needs to update task, **When** using `TaskUpdate` tool, **Then** updates status/description
3. **Given** Agent needs to list tasks, **When** using `TaskList` tool, **Then** returns all tasks
4. **Given** Agent needs to get task details, **When** using `TaskGet` tool, **Then** returns task details
5. **Given** Agent needs to stop task, **When** using `TaskStop` tool, **Then** terminates running task
6. **Given** Agent needs to get task output, **When** using `TaskOutput` tool, **Then** returns task execution output

### User Story 3 - Planning & Worktree Tools (Phase 2)

**Acceptance Scenarios**:

1. **Given** Agent needs to enter plan mode, **When** using `EnterPlanMode` tool, **Then** switches to plan mode
2. **Given** Agent needs to exit plan mode, **When** using `ExitPlanMode` tool, **Then** returns to execution mode
3. **Given** Agent needs to enter worktree, **When** using `EnterWorktree` tool, **Then** creates or enters git worktree
4. **Given** Agent needs to exit worktree, **When** using `ExitWorktree` tool, **Then** returns to main workspace

### User Story 4 - MCP Integration Tools (Phase 2)

**Acceptance Scenarios**:

1. **Given** Agent needs to call MCP tool, **When** using `MCPTool` tool, **Then** calls MCP server and returns result
2. **Given** Agent needs to list MCP resources, **When** using `ListMcpResources` tool, **Then** returns available resources
3. **Given** Agent needs to read MCP resource, **When** using `ReadMcpResource` tool, **Then** returns resource content
4. **Given** Agent needs MCP authentication, **When** using `McpAuth` tool, **Then** completes authentication flow

### User Story 5 - Config & Skills Tools (Phase 2)

**Acceptance Scenarios**:

1. **Given** Agent needs to view config, **When** using `ConfigTool` tool, **Then** returns current config items
2. **Given** Agent needs to modify config, **When** using `ConfigTool` tool, **Then** updates and persists config
3. **Given** Agent needs to discover skills, **When** using `DiscoverSkills` tool, **Then** returns skill list
4. **Given** Agent needs to execute skill, **When** using `SkillTool` tool, **Then** executes specified skill
5. **Given** Agent needs brief output, **When** using `Brief` tool, **Then** switches to brief mode
6. **Given** Agent needs to inspect context, **When** using `CtxInspect` tool, **Then** shows current context state

### User Story 6 - Enhanced Tools (Phase 3)

**Acceptance Scenarios**:

1. **Given** Agent needs to ask user, **When** using `AskUserQuestion` tool, **Then** shows question and waits
2. **Given** Agent needs to browse web, **When** using `WebBrowser` tool, **Then** renders and interacts with page
3. **Given** Agent needs to take screenshot, **When** using `SnipTool` tool, **Then** captures screen content
4. **Given** Agent needs synthetic output, **When** using `SyntheticOutput` tool, **Then** generates structured output
5. **Given** Agent needs to review artifact, **When** using `ReviewArtifact` tool, **Then** provides review interface
6. **Given** Agent needs push notification, **When** using `PushNotification` tool, **Then** sends notification
7. **Given** Agent needs terminal capture, **When** using `TerminalCapture` tool, **Then** captures terminal output
8. **Given** Agent needs to delay, **When** using `SleepTool` tool, **Then** waits specified time
9. **Given** Agent needs to search tools, **When** using `ToolSearch` tool, **Then** returns matching tools

### Edge Cases

- Tool execution timeout: MUST return timeout error with clear message
- Invalid tool parameters: MUST return validation error with field details
- Tool execution interrupted: MUST gracefully stop and return interruption status
- Insufficient permissions: MUST return permission error (not silent failure)
- MCP server unavailable: MUST return connection error
- Git worktree operation failure: MUST provide clear error message
- Circular dependency detected: MUST return dependency cycle error

## Requirements

### Functional Requirements

#### Phase 1 - Core Foundation (17 tools)

| ID | Requirement | Tool | Priority |
|----|-------------|------|----------|
| FR-101 | MUST implement `BashTool` with shell command execution | C01 | MUST |
| FR-102 | MUST implement `GlobTool` with pattern-based file search | C03 | MUST |
| FR-103 | MUST implement `GrepTool` with regex content search | C02 | MUST |
| FR-104 | MUST implement `ReadTool` for file content reading | C04 | MUST |
| FR-105 | MUST implement `EditTool` for string replacement editing | C05 | MUST |
| FR-106 | MUST implement `WriteTool` for file creation/overwrite | C06 | MUST |
| FR-107 | MUST implement `NotebookEditTool` for .ipynb JSON editing | C07 | MUST |
| FR-108 | MUST implement `WebFetchTool` for webpage content retrieval | C08 | MUST |
| FR-109 | MUST implement `WebSearchTool` for web search | C09 | MUST |
| FR-110 | MUST implement `TodoWriteTool` for task list management | C10 | MUST |
| FR-111 | MUST implement `AgentTool` for sub-agent spawning | C11 | MUST |
| FR-112 | MUST implement `TaskCreateTool` creating task with unique ID | T01 | MUST |
| FR-113 | MUST implement `TaskUpdateTool` updating task status/priority | T02 | MUST |
| FR-114 | MUST implement `TaskListTool` listing all tasks | T03 | MUST |
| FR-115 | MUST implement `TaskGetTool` getting task details | T04 | MUST |
| FR-116 | MUST implement `TaskStopTool` terminating running task | T05 | MUST |
| FR-117 | MUST implement `TaskOutputTool` getting task execution output | T06 | MUST |

#### Phase 2 - System Integration (13 tools)

| ID | Requirement | Tool | Priority |
|----|-------------|------|----------|
| FR-201 | MUST implement `EnterPlanModeTool` switching to plan mode | W01 | MUST |
| FR-202 | MUST implement `ExitPlanModeTool` exiting plan mode | W02 | MUST |
| FR-203 | MUST implement `EnterWorktreeTool` creating/entering git worktree | W03 | MUST |
| FR-204 | MUST implement `ExitWorktreeTool` exiting worktree | W04 | MUST |
| FR-301 | MUST implement `MCPTool` calling MCP server tools | M01 | MUST |
| FR-302 | MUST implement `ListMcpResourcesTool` listing MCP resources | M02 | MUST |
| FR-303 | MUST implement `ReadMcpResourceTool` reading MCP resource | M03 | MUST |
| FR-304 | MUST implement `McpAuthTool` handling MCP authentication | M04 | MUST |
| FR-401 | MUST implement `ConfigTool` viewing and modifying config | G01 | MUST |
| FR-402 | MUST implement `SkillTool` executing specified skill | G02 | MUST |
| FR-403 | MUST implement `DiscoverSkillsTool` discovering available skills | G03 | MUST |
| FR-404 | MUST implement `BriefTool` switching to brief output mode | G04 | SHOULD |
| FR-405 | MUST implement `CtxInspectTool` inspecting current context | G05 | SHOULD |

#### Phase 3 - Enhanced Features (22 tools)

| ID | Requirement | Tool | Priority |
|----|-------------|------|----------|
| FR-501 | MUST implement `LSPTool` for Language Server Protocol | L01 | MUST |
| FR-601 | MUST implement `CronCreateTool` creating scheduled task | S01 | MUST |
| FR-602 | MUST implement `CronDeleteTool` deleting scheduled task | S02 | MUST |
| FR-603 | MUST implement `CronListTool` listing scheduled tasks | S03 | MUST |
| FR-604 | MUST implement `WorkflowTool` executing defined workflow | S04 | MUST |
| FR-701 | MUST implement `SendMessageTool` sending message to target | P01 | MUST |
| FR-702 | MUST implement `ListPeersTool` listing team members | P02 | MUST |
| FR-703 | MUST implement `TeamCreateTool` creating team | P03 | MUST |
| FR-704 | MUST implement `TeamDeleteTool` deleting team | P04 | MUST |
| FR-705 | MUST implement `RemoteTriggerTool` remotely triggering operation | P05 | MUST |
| FR-801 | MUST implement `AskUserQuestionTool` asking user question | E01 | MUST |
| FR-802 | MUST implement `WebBrowserTool` browsing and interacting with web | E02 | MUST |
| FR-803 | MUST implement `SnipTool` screen capture | E03 | SHOULD |
| FR-804 | MUST implement `SyntheticOutputTool` generating synthetic output | E04 | SHOULD |
| FR-805 | MUST implement `ReviewArtifactTool` reviewing artifacts | E05 | SHOULD |
| FR-806 | MUST implement `SubscribePRTool` subscribing to PR events | E06 | SHOULD |
| FR-807 | MUST implement `SuggestBackgroundPRTool` suggesting background PR | E07 | SHOULD |
| FR-808 | MUST implement `PushNotificationTool` sending push notification | E08 | SHOULD |
| FR-809 | MUST implement `TerminalCaptureTool` capturing terminal output | E09 | SHOULD |
| FR-810 | MUST implement `MonitorTool` monitoring system state | E10 | SHOULD |
| FR-811 | MUST implement `SleepTool` delaying execution | E11 | SHOULD |
| FR-812 | MUST implement `ToolSearchTool` searching available tools | E12 | SHOULD |

#### Cross-Cutting Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| FR-001 | All tools MUST support timeout control (default 5 minutes) | MUST |
| FR-002 | All tools MUST support cancellation (Ctrl+C interrupt) | MUST |
| FR-003 | All write operations MUST support atomic write with auto-backup | MUST |
| FR-004 | All file operations MUST correctly handle paths (relative, absolute, ~ expansion) | MUST |
| FR-005 | Bash tool MUST support background execution mode (`run_in_background`) | MUST |
| FR-006 | System MUST implement tool execution permission checking | MUST |
| FR-007 | All tools MUST log execution events (invoke, complete, fail) | MUST |
| FR-008 | All tools MUST return consistent `ToolResult` structure | MUST |
| FR-009 | All tools MUST validate input against schema before execution | MUST |
| FR-010 | Tool execution MUST track resource usage (memory, CPU time) | SHOULD |

#### Enhanced Capabilities (超越 Claude Code)

| ID | Capability | Status |
|----|------------|--------|
| EC-101 | WebFetch SHOULD support CSS selector extraction | ✅ Implemented |
| EC-102 | Bash SHOULD support command history and auto-completion | 🔄 Planned |
| EC-103 | Read SHOULD support syntax highlighting for Markdown and code | 🔄 Planned |
| EC-104 | Glob SHOULD support exclude patterns | 🔄 Planned |
| EC-105 | System SHOULD support streaming output for tool results | 🔄 Planned |

### Non-Functional Requirements

| Category | Requirement | Target |
|----------|-------------|--------|
| Performance | Tool execution average response time (excluding actual command) | < 100ms |
| Reliability | Write operations must support atomic write | 100% |
| Availability | Tool help information completeness | >= 95% |
| Compatibility | Tool API version compatibility | Backward compatible |

## Key Entities

| Entity | Description | Fields |
|--------|-------------|--------|
| Tool | Tool definition with name, description, schema, execution logic | name, description, version, schema, execute_fn |
| ToolExecution | Tool execution record | id, tool_name, input, output, duration_ms, status |
| ToolPermission | Tool permission configuration | tool_name, permission_level, required_capabilities |
| ToolResult | Tool execution result | success, data, error, metadata |
| ToolDependency | Tool dependency relationship | tool_a, tool_b, dependency_type |
| Task | Task entity | id, title, description, status, created_at, updated_at |
| Cron | Scheduled task entity | id, expression, command, enabled, last_run, next_run |
| Workflow | Workflow entity | id, name, steps, status |
| Team/Peer | Team member entity | id, name, role, status |
| McpServer | MCP server entity | id, name, tools, resources, auth |
| Skill | Skill entity | id, name, description, execute_fn |

## Success Criteria

| ID | Criterion | Metric | Current Status |
|----|-----------|--------|----------------|
| SC-001 | Claude Code tool implementation | 52/52 tools (100%) | 10/52 (19%) |
| SC-002 | Completed tools test pass rate | >= 3 test cases per tool | N/A |
| SC-003 | Tool execution average response time | < 100ms | N/A |
| SC-004 | Atomic write support for all write operations | 100% | N/A |
| SC-005 | Tool help information completeness | >= 95% | N/A |
| SC-006 | Complete task lifecycle support | All 6 task tools | N/A |

## Implementation Verification Matrix

### Tool Count Verification

| Category | Expected | Implemented | Verified | Gap |
|----------|----------|-------------|----------|-----|
| Core | 11 | 10 | ✅ | 1 (NotebookEdit) |
| Task | 6 | 0 | ✅ | 6 |
| MCP | 4 | 0 | ✅ | 4 |
| Config | 5 | 0 | ✅ | 5 |
| LSP | 1 | 0 | ✅ | 1 |
| Schedule | 4 | 0 | ✅ | 4 |
| Communication | 5 | 0 | ✅ | 5 |
| Enhanced | 12 | 0 | ✅ | 12 |
| Planning | 4 | 0 | ✅ | 4 |
| **Total** | **52** | **10** | ✅ | **42** |

### Dependency Verification Checklist

- [ ] No circular dependencies in tool execution
- [ ] All dependencies declared in tool metadata
- [ ] Dependency validation performed before tool execution
- [ ] Missing dependency returns clear error message
- [ ] Optional dependencies handled gracefully

### Version Management Checklist

- [ ] All tools have version numbers in format `major.minor.patch`
- [ ] Version changes documented in changelog
- [ ] Breaking changes increment major version
- [ ] New features increment minor version
- [ ] Bug fixes increment patch version

## Assumptions

- User has stable network connection (for WebFetch/WebSearch/MCP)
- File system supports standard Unix path semantics
- Git is installed and available (for Worktree tools)
- LSP servers can communicate via standard protocol
- Target is CLI tool, GUI interaction out of scope
- MCP servers require separate deployment and configuration
- Some tools (PowerShell, WebBrowser) may require platform-specific implementation

## Clarifications

### Session 2026-04-18

- Q: 规范 003 和 004 合并 → A: 合并为单一规范，003 已实现的 10 个工具标记为完成，004 的 43 个工具按优先级分阶段实现

### Session 2026-04-21

- Q: 53 tools vs actual count → A: 实际清点为 52 tools，已修正分类统计
- Q: 工具数量验证 → A: 添加了 Implementation Verification Matrix
- Q: 工具依赖关系定义 → A: 添加了 Tool Dependencies 章节，包含依赖矩阵
- Q: 工具版本管理 → A: 添加了 Tool Version Management 章节
- Q: 统一接口规范 → A: 按 SPEC_DEPENDENCIES.md 规范添加了 Uniform Interface 章节

(End of file - total lines: ~520)
