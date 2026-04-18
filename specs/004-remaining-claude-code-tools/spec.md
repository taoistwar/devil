# Feature Specification: Claude Code Remaining Tools Implementation

**Feature Branch**: `260418-feat-claude-code-tools-alignment`  
**Created**: 2026-04-18  
**Status**: Draft  
**Input**: "将上面列出的所有工具，逐个和当前项目的做对比，当前项目的要对齐它们。它们原来是ts（typescript），当前项目还用rust编写而已"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Core File & Shell Tools (Priority: P1)

实现核心文件操作和 Shell 工具的完整对齐。

**Why this priority**: 这些是 Agent 与系统交互的基础工具，必须完整实现。

**Independent Test**: 运行 `devil run "使用 NotebookEdit 编辑 .ipynb 文件"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要编辑 Jupyter notebook, **When** 使用 `NotebookEdit` 工具, **Then** 正确解析并修改 .ipynb 文件
2. **Given** Agent 需要进入交互模式, **When** 使用 `REPL` 工具, **Then** 启动交互式 REPL 会话
3. **Given** Agent 需要在 Windows 执行命令, **When** 使用 `PowerShell` 工具, **Then** 通过 PowerShell 执行命令

---

### User Story 2 - Planning & Worktree Tools (Priority: P1)

实现规划和 Git worktree 管理工具。

**Why this priority**: 规划模式是 Claude Code 的核心特性，worktree 支持多分支并行工作。

**Independent Test**: 运行 `devil run "使用 EnterPlanMode 进入规划模式"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要进入规划模式, **When** 使用 `EnterPlanMode` 工具, **Then** 切换到规划模式状态
2. **Given** Agent 需要退出规划模式, **When** 使用 `ExitPlanMode` 工具, **Then** 恢复到正常执行模式
3. **Given** Agent 需要在独立分支工作, **When** 使用 `EnterWorktree` 工具, **Then** 创建或进入 git worktree
4. **Given** Agent 需要退出 worktree, **When** 使用 `ExitWorktree` 工具, **Then** 返回主工作区

---

### User Story 3 - Task Management Tools (Priority: P1)

实现任务创建、更新、查询、停止完整生命周期管理。

**Why this priority**: 任务管理是复杂任务分解和跟踪的基础机制。

**Independent Test**: 运行 `devil run "使用 TaskCreate 创建任务，使用 TaskList 查看任务"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要创建任务, **When** 使用 `TaskCreate` 工具, **Then** 创建新任务并返回 task_id
2. **Given** Agent 需要更新任务状态, **When** 使用 `TaskUpdate` 工具, **Then** 更新任务状态/描述
3. **Given** Agent 需要查看任务, **When** 使用 `TaskList` 工具, **Then** 返回所有任务列表
4. **Given** Agent 需要获取任务详情, **When** 使用 `TaskGet` 工具, **Then** 返回指定任务详情
5. **Given** Agent 需要停止任务, **When** 使用 `TaskStop` 工具, **Then** 终止正在执行的任务

---

### User Story 4 - MCP Integration Tools (Priority: P1)

实现 MCP (Model Context Protocol) 完整集成。

**Why this priority**: MCP 是扩展工具能力的标准协议，必须完整支持。

**Independent Test**: 运行 `devil run "使用 MCPTool 调用 MCP 服务器工具"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要调用 MCP 工具, **When** 使用 `MCPTool` 工具, **Then** 正确调用 MCP 服务器并返回结果
2. **Given** Agent 需要查看 MCP 资源, **When** 使用 `ListMcpResources` 工具, **Then** 返回可用资源列表
3. **Given** Agent 需要读取 MCP 资源, **When** 使用 `ReadMcpResource` 工具, **Then** 返回资源内容
4. **Given** Agent 需要 MCP 认证, **When** 使用 `McpAuth` 工具, **Then** 完成 MCP 认证流程

---

### User Story 5 - Configuration & Skills Tools (Priority: P2)

实现配置管理和技能发现系统。

**Why this priority**: 配置管理和技能系统提供可扩展性和用户定制能力。

**Independent Test**: 运行 `devil run "使用 ConfigTool 查看配置，使用 SkillTool 列出技能"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要查看配置, **When** 使用 `ConfigTool` 工具, **Then** 返回当前配置项
2. **Given** Agent 需要修改配置, **When** 使用 `ConfigTool` 设置配置, **Then** 更新配置并持久化
3. **Given** Agent 需要发现可用技能, **When** 使用 `DiscoverSkills` 工具, **Then** 返回技能列表和使用说明
4. **Given** Agent 需要执行技能, **When** 使用 `SkillTool` 工具, **Then** 调用指定技能执行

---

### User Story 6 - LSP Language Server Tools (Priority: P2)

实现语言服务器协议集成。

**Why this priority**: LSP 提供代码补全、跳转、诊断等 IDE 级别的功能。

**Independent Test**: 运行 `devil run "使用 LSPTool 跳转到函数定义"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要代码补全, **When** 使用 `LSPTool` 请求补全, **Then** 返回补全候选项
2. **Given** Agent 需要跳转到定义, **When** 使用 `LSPTool` 跳转请求, **Then** 返回定义位置
3. **Given** Agent 需要代码诊断, **When** 使用 `LSPTool` 诊断请求, **Then** 返回诊断信息

---

### User Story 7 - Scheduling & Workflow Tools (Priority: P2)

实现定时任务和工作流管理。

**Why this priority**: 定时任务和工作流实现后台自动化处理能力。

**Independent Test**: 运行 `devil run "使用 CronCreate 创建定时任务，使用 CronList 查看"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要创建定时任务, **When** 使用 `CronCreate` 工具, **Then** 创建 cron 并返回 task_id
2. **Given** Agent 需要删除定时任务, **When** 使用 `CronDelete` 工具, **Then** 删除指定的 cron
3. **Given** Agent 需要列出定时任务, **When** 使用 `CronList` 工具, **Then** 返回所有 cron 任务
4. **Given** Agent 需要执行工作流, **When** 使用 `WorkflowTool` 工具, **Then** 按定义执行工作流步骤

---

### User Story 8 - Communication & Team Tools (Priority: P2)

实现消息发送和团队协作工具。

**Why this priority**: 消息和团队工具支持多 Agent 协作场景。

**Independent Test**: 运行 `devil run "使用 SendMessage 发送消息，使用 ListPeers 查看 peers"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要发送消息, **When** 使用 `SendMessage` 工具, **Then** 发送消息到指定目标
2. **Given** Agent 需要查看团队成员, **When** 使用 `ListPeers` 工具, **Then** 返回团队成员列表
3. **Given** Agent 需要创建团队, **When** 使用 `TeamCreate` 工具, **Then** 创建新团队
4. **Given** Agent 需要删除团队, **When** 使用 `TeamDelete` 工具, **Then** 删除指定团队

---

### User Story 9 - Enhanced Tools (Priority: P3)

实现增强型工具集。

**Why this priority**: 增强工具提供更高级的用户交互和输出能力。

**Independent Test**: 运行 `devil run "使用 AskUserQuestion 询问用户，使用 WebBrowser 浏览网页"` 并验证

**Acceptance Scenarios**:

1. **Given** Agent 需要询问用户, **When** 使用 `AskUserQuestion` 工具, **Then** 显示问题并等待用户回答
2. **Given** Agent 需要浏览网页, **When** 使用 `WebBrowser` 工具, **Then** 渲染并交互网页
3. **Given** Agent 需要截图, **When** 使用 `SnipTool` 工具, **Then** 捕获屏幕内容
4. **Given** Agent 需要合成输出, **When** 使用 `SyntheticOutput` 工具, **Then** 生成结构化输出
5. **Given** Agent 需要审查产物, **When** 使用 `ReviewArtifact` 工具, **Then** 提供产物审查界面
6. **Given** Agent 需要推送通知, **When** 使用 `PushNotification` 工具, **Then** 发送推送通知
7. **Given** Agent 需要捕获终端, **When** 使用 `TerminalCapture` 工具, **Then** 捕获终端输出
8. **Given** Agent 需要延迟执行, **When** 使用 `SleepTool` 工具, **Then** 等待指定时间

---

## Requirements *(mandatory)*

### Functional Requirements

#### Core File & Shell Tools (FR-100s)
- **FR-101**: 系统 MUST 实现 `NotebookEditTool`，支持 .ipynb 文件的 JSON 格式解析和编辑
- **FR-102**: 系统 MUST 实现 `REPLTool`，支持交互式输入输出会话
- **FR-103**: 系统 MUST 实现 `PowerShellTool`，支持 Windows PowerShell 命令执行

#### Planning & Worktree Tools (FR-200s)
- **FR-201**: 系统 MUST 实现 `EnterPlanModeTool`，切换到规划模式状态
- **FR-202**: 系统 MUST 实现 `ExitPlanModeTool`，退出规划模式恢复到执行模式
- **FR-203**: 系统 MUST 实现 `EnterWorktreeTool`，支持 git worktree 创建和进入
- **FR-204**: 系统 MUST 实现 `ExitWorktreeTool`，退出 worktree 返回主工作区

#### Task Management Tools (FR-300s)
- **FR-301**: 系统 MUST 实现 `TaskCreateTool`，创建新任务并返回唯一 task_id
- **FR-302**: 系统 MUST 实现 `TaskUpdateTool`，更新任务状态、描述、优先级
- **FR-303**: 系统 MUST 实现 `TaskListTool`，列出所有任务及状态
- **FR-304**: 系统 MUST 实现 `TaskGetTool`，获取指定任务的完整信息
- **FR-305**: 系统 MUST 实现 `TaskStopTool`，停止正在执行的任务
- **FR-306**: 系统 MUST 实现 `TaskOutputTool`，获取任务执行的输出

#### MCP Integration Tools (FR-400s)
- **FR-401**: 系统 MUST 实现 `MCPTool`，支持调用 MCP 服务器工具
- **FR-402**: 系统 MUST 实现 `ListMcpResourcesTool`，列出 MCP 服务器可用资源
- **FR-403**: 系统 MUST 实现 `ReadMcpResourceTool`，读取指定 MCP 资源内容
- **FR-404**: 系统 MUST 实现 `McpAuthTool`，处理 MCP 服务器认证

#### Configuration & Skills Tools (FR-500s)
- **FR-501**: 系统 MUST 实现 `ConfigTool`，支持配置的查看和修改
- **FR-502**: 系统 MUST 实现 `SkillTool`，执行指定的 skill
- **FR-503**: 系统 MUST 实现 `DiscoverSkillsTool`，发现和列出可用技能
- **FR-504**: 系统 MUST 实现 `BriefTool`，切换到简洁输出模式
- **FR-505**: 系统 MUST 实现 `CtxInspectTool`，检查和展示当前上下文状态

#### LSP Tools (FR-600s)
- **FR-601**: 系统 MUST 实现 `LSPTool`，支持语言服务器协议通信

#### Scheduling & Workflow Tools (FR-700s)
- **FR-701**: 系统 MUST 实现 `CronCreateTool`，创建定时任务
- **FR-702**: 系统 MUST 实现 `CronDeleteTool`，删除定时任务
- **FR-703**: 系统 MUST 实现 `CronListTool`，列出所有定时任务
- **FR-704**: 系统 MUST 实现 `WorkflowTool`，执行定义的工作流

#### Communication & Team Tools (FR-800s)
- **FR-801**: 系统 MUST 实现 `SendMessageTool`，发送消息到指定目标
- **FR-802**: 系统 MUST 实现 `ListPeersTool`，列出团队成员/peers
- **FR-803**: 系统 MUST 实现 `TeamCreateTool`，创建团队
- **FR-804**: 系统 MUST 实现 `TeamDeleteTool`，删除团队
- **FR-805**: 系统 MUST 实现 `RemoteTriggerTool`，远程触发操作

#### Enhanced Tools (FR-900s)
- **FR-901**: 系统 MUST 实现 `AskUserQuestionTool`，向用户提问并等待回答
- **FR-902**: 系统 MUST 实现 `WebBrowserTool`，浏览和交互网页
- **FR-903**: 系统 MUST 实现 `SnipTool`，屏幕截图
- **FR-904**: 系统 MUST 实现 `SyntheticOutputTool`，生成合成输出
- **FR-905**: 系统 MUST 实现 `ReviewArtifactTool`，审查 artifacts
- **FR-906**: 系统 MUST 实现 `SubscribePRTool`，订阅 PR 事件
- **FR-907**: 系统 MUST 实现 `SuggestBackgroundPRTool`，建议后台处理 PR
- **FR-908**: 系统 MUST 实现 `PushNotificationTool`，发送推送通知
- **FR-909**: 系统 MUST 实现 `TerminalCaptureTool`，捕获终端输出
- **FR-910**: 系统 MUST 实现 `MonitorTool`，监控系统状态
- **FR-911**: 系统 MUST 实现 `SleepTool`，延迟执行
- **FR-912**: 系统 SHOULD 实现 `ToolSearchTool`，搜索可用工具

### Key Entities

- **Task**: 任务实体，包含 id, title, description, status, created_at, updated_at
- **Cron**: 定时任务实体，包含 id, expression, command, enabled, last_run, next_run
- **Workflow**: 工作流实体，包含 id, name, steps, status
- **Team/Peer**: 团队成员实体，包含 id, name, role, status
- **McpServer**: MCP 服务器实体，包含 id, name, tools, resources, auth
- **Skill**: 技能实体，包含 id, name, description, execute_fn

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 所有 43 个缺失工具 MUST 完成实现
- **SC-002**: 每个工具 MUST 通过至少 3 个测试用例（正常、边界、错误）
- **SC-003**: 工具执行平均响应时间 < 100ms（不含实际命令执行）
- **SC-004**: 工具帮助信息完整度 >= 95%
- **SC-005**: 所有任务相关工具 MUST 支持完整任务生命周期

## Assumptions

- 用户有稳定网络连接（用于 MCP、WebBrowser 等外部交互工具）
- 文件系统支持标准 Unix 路径语义
- Git 已安装并可用（用于 Worktree 工具）
- LSP 服务器可通过标准协议通信
- 目标是 CLI 工具，GUI 交互不在范围内
- MCP 服务器需要单独部署和配置
- 部分工具（如 PowerShell、WebBrowser）可能需要平台特定实现

## Implementation Priority

### Phase 1 (P1 - Core)
1. Task management tools (TaskCreate, TaskUpdate, TaskList, TaskGet, TaskStop)
2. Planning tools (EnterPlanMode, ExitPlanMode)
3. Worktree tools (EnterWorktree, ExitWorktree)
4. MCP tools (MCPTool, ListMcpResources, ReadMcpResource, McpAuth)

### Phase 2 (P2 - Enhanced)
5. Configuration tools (ConfigTool, BriefTool, CtxInspectTool)
6. Skills tools (SkillTool, DiscoverSkillsTool)
7. Scheduling tools (CronCreate, CronDelete, CronList)
8. Workflow tools (WorkflowTool)
9. Communication tools (SendMessage, ListPeers, TeamCreate, TeamDelete)

### Phase 3 (P3 - Advanced)
10. File tools (NotebookEditTool, REPLTool, PowerShellTool)
11. LSP tool
12. Enhanced tools (WebBrowser, SnipTool, SyntheticOutput, etc.)
13. Tool search (ToolSearchTool)
