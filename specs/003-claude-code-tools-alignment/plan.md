# Implementation Plan: Claude Code Tools Alignment

**Branch**: `260418-feat-claude-code-tools-alignment` | **Date**: 2026-04-18 | **Spec**: [spec.md](./spec.md)

## Summary

实现 Claude Code 全部 53 个内置工具的对齐。当前项目已实现 10 个核心工具，剩余 43 个工具需要分 3 个优先级阶段完成。所有工具遵循 Rust-First 标准、Tokio 并发模型和 Claude Code 五要素协议。

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: tokio, async-trait, serde, reqwest, regex, glob, walkdir, dashmap  
**Storage**: 任务/定时任务状态存储在内存中，持久化到文件系统  
**Testing**: cargo test, 单元测试 + 集成测试  
**Target Platform**: Linux server environments  
**Project Type**: CLI tool library (devil-agent)  
**Performance Goals**: 工具执行响应 < 100ms（不含实际命令执行）  
**Constraints**: 遵循 Claude Code 五要素协议 (Input, Output, Progress, Permissions, Metadata)  
**Scale/Scope**: 53 个工具（10 个已完成，43 个待实现）

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| Rust-First Standards | ✅ PASS | 所有代码使用 idiomatic Rust |
| Tokio Concurrency Model | ✅ PASS | 使用 `#[async_trait]` 和 Tokio runtime |
| Claude Code Reference Parity | ✅ PASS | 工具名称、参数与参考实现一致 |
| Robust Error Handling | ✅ PASS | 使用 `anyhow::Result<T>` 和 `thiserror` |
| Tool-First Architecture | ✅ PASS | 所有功能通过工具暴露 |

## Completed Implementation (10/53 tools) ✅

### Phase 1: Core Tools (DONE)
| 工具 | 状态 | 实现文件 |
|------|------|----------|
| BashTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileReadTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileEditTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| FileWriteTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| GlobTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| GrepTool | ✅ | `crates/agent/src/tools/builtin.rs` |

### Phase 2: Web & Task Tools (DONE)
| 工具 | 状态 | 实现文件 |
|------|------|----------|
| WebFetchTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| WebSearchTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| TodoWriteTool | ✅ | `crates/agent/src/tools/builtin.rs` |
| AgentTool | ✅ | `crates/agent/src/tools/builtin.rs` |

### Enhanced Features (DONE)
| 功能 | 状态 | 实现文件 |
|------|------|----------|
| WebFetch CSS Selector | ✅ | `crates/agent/src/tools/builtin.rs` |

## Phase 1: Core Infrastructure (P1) - 13 tools

### 1.1 Task Management Tools (6 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| TaskCreateTool | 创建任务 | `TaskCreateTool/TaskCreateTool.ts` |
| TaskUpdateTool | 更新任务状态 | `TaskUpdateTool/TaskUpdateTool.ts` |
| TaskListTool | 列出所有任务 | `TaskListTool/TaskListTool.ts` |
| TaskGetTool | 获取任务详情 | `TaskGetTool/TaskGetTool.ts` |
| TaskStopTool | 停止任务 | `TaskStopTool/TaskStopTool.ts` |
| TaskOutputTool | 获取任务输出 | `TaskOutputTool/constants.ts` |

**实现要点**:
- 任务状态机: `pending` → `in_progress` → `completed` / `failed` / `stopped`
- 使用 `DashMap` 存储任务状态
- 任务 ID 使用 UUID
- 实现位置: `crates/agent/src/tools/task_tools.rs`

### 1.2 Planning Mode Tools (2 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| EnterPlanModeTool | 进入规划模式 | `EnterPlanModeTool/EnterPlanModeTool.ts` |
| ExitPlanModeTool | 退出规划模式 | `ExitPlanModeTool/ExitPlanModeV2Tool.ts` |

**实现要点**:
- Agent 状态机增加 `planning` 状态
- 规划模式使用不同的 system prompt
- 实现位置: `crates/agent/src/tools/planning_tools.rs`

### 1.3 Worktree Tools (2 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| EnterWorktreeTool | 进入/创建 worktree | `EnterWorktreeTool/EnterWorktreeTool.ts` |
| ExitWorktreeTool | 退出 worktree | `ExitWorktreeTool/ExitWorktreeTool.ts` |

**实现要点**:
- 使用 `git worktree` 命令
- 跟踪多个工作目录状态
- 实现位置: `crates/agent/src/tools/worktree_tools.rs`

### 1.4 MCP Integration Tools (4 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| MCPTool | 调用 MCP 工具 | `MCPTool/MCPTool.ts` |
| ListMcpResourcesTool | 列出 MCP 资源 | `ListMcpResourcesTool/ListMcpResourcesTool.ts` |
| ReadMcpResourceTool | 读取 MCP 资源 | `ReadMcpResourceTool/ReadMcpResourceTool.ts` |
| McpAuthTool | MCP 认证 | `McpAuthTool/McpAuthTool.ts` |

**实现要点**:
- 集成现有 `crates/mcp` 模块
- 支持 MCP 协议 JSON-RPC 通信
- 实现位置: `crates/agent/src/tools/mcp_tools.rs`

## Phase 2: Enhanced Tools (P2) - 16 tools

### 2.1 Configuration Tools (3 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| ConfigTool | 查看/修改配置 | `ConfigTool/ConfigTool.ts` |
| BriefTool | 简洁模式 | `BriefTool/BriefTool.ts` |
| CtxInspectTool | 上下文检查 | `CtxInspectTool/CtxInspectTool.ts` |

### 2.2 Skills Tools (2 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| SkillTool | 执行技能 | `SkillTool/SkillTool.ts` |
| DiscoverSkillsTool | 发现技能 | `DiscoverSkillsTool/prompt.ts` |

### 2.3 Scheduling Tools (3 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| CronCreateTool | 创建定时任务 | `ScheduleCronTool/CronCreateTool.ts` |
| CronDeleteTool | 删除定时任务 | `ScheduleCronTool/CronListTool.ts` |
| CronListTool | 列出定时任务 | `ScheduleCronTool/CronDeleteTool.ts` |

**实现要点**:
- 使用 `tokio::time::interval` 实现调度
- 持久化 cron 表达式到配置文件
- 实现位置: `crates/agent/src/tools/scheduling_tools.rs`

### 2.4 Workflow Tools (1 tool)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| WorkflowTool | 执行工作流 | `WorkflowTool/createWorkflowCommand.ts` |

### 2.5 Communication Tools (4 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| SendMessageTool | 发送消息 | `SendMessageTool/SendMessageTool.ts` |
| ListPeersTool | 列出团队成员 | `ListPeersTool/ListPeersTool.ts` |
| TeamCreateTool | 创建团队 | `TeamCreateTool/TeamCreateTool.ts` |
| TeamDeleteTool | 删除团队 | `TeamDeleteTool/TeamDeleteTool.ts` |

## Phase 3: Advanced Tools (P3) - 14 tools

### 3.1 File Tools (3 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| NotebookEditTool | 编辑 Jupyter notebook | `NotebookEditTool/NotebookEditTool.ts` |
| REPLTool | 交互式 REPL | `REPLTool/REPLTool.ts` |
| PowerShellTool | PowerShell 执行 | `PowerShellTool/prompt.ts` |

### 3.2 LSP Tool (1 tool)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| LSPTool | 语言服务器协议 | `LSPTool/prompt.ts` |

### 3.3 Enhanced Tools (10 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| AskUserQuestionTool | 询问用户 | `AskUserQuestionTool/prompt.ts` |
| WebBrowserTool | 网页浏览 | `WebBrowserTool/WebBrowserTool.ts` |
| SnipTool | 截图 | `SnipTool/SnipTool.ts` |
| SyntheticOutputTool | 合成输出 | `SyntheticOutputTool/SyntheticOutputTool.ts` |
| ReviewArtifactTool | 审查产物 | `ReviewArtifactTool/ReviewArtifactTool.ts` |
| SubscribePRTool | 订阅 PR | `SubscribePRTool/SubscribePRTool.ts` |
| SuggestBackgroundPRTool | 建议后台 PR | `SuggestBackgroundPRTool/SuggestBackgroundPRTool.ts` |
| PushNotificationTool | 推送通知 | `PushNotificationTool/PushNotificationTool.ts` |
| TerminalCaptureTool | 终端捕获 | `TerminalCaptureTool/TerminalCaptureTool.ts` |
| MonitorTool | 监控 | (空目录 - stub) |
| SleepTool | 延迟执行 | `SleepTool/SleepTool.ts` |
| ToolSearchTool | 搜索工具 | `ToolSearchTool/ToolSearchTool.ts` |
| RemoteTriggerTool | 远程触发 | `RemoteTriggerTool/RemoteTriggerTool.ts` |

## Project Structure

```text
crates/agent/src/
├── tools/
│   ├── builtin.rs              # 10 个已完成工具
│   ├── task_tools.rs           # Phase 1: Task tools (6)
│   ├── planning_tools.rs        # Phase 1: Planning tools (2)
│   ├── worktree_tools.rs       # Phase 1: Worktree tools (2)
│   ├── mcp_tools.rs            # Phase 1: MCP tools (4)
│   ├── config_tools.rs         # Phase 2: Config tools (3)
│   ├── skills_tools.rs          # Phase 2: Skills tools (2)
│   ├── scheduling_tools.rs      # Phase 2: Scheduling tools (3)
│   ├── workflow_tools.rs       # Phase 2: Workflow tools (1)
│   ├── comm_tools.rs           # Phase 2: Communication tools (4)
│   ├── file_tools.rs           # Phase 3: File tools (3)
│   ├── lsp_tools.rs            # Phase 3: LSP tool (1)
│   ├── enhanced_tools.rs       # Phase 3: Enhanced tools (12)
│   ├── tool.rs                 # Tool trait 定义
│   ├── registry.rs             # 工具注册表
│   ├── executor.rs             # 工具执行器
│   └── build_tool.rs           # 工具构建器
├── subagent/
│   └── mod.rs                  # 子代理模块
└── core.rs                     # Agent 主模块
```

## Implementation Order

```
Phase 1 (P1 - Core Infrastructure):
  1. TaskCreateTool → TaskUpdateTool → TaskListTool → TaskGetTool → TaskStopTool → TaskOutputTool
  2. EnterPlanModeTool → ExitPlanModeTool
  3. EnterWorktreeTool → ExitWorktreeTool
  4. MCPTool → ListMcpResourcesTool → ReadMcpResourceTool → McpAuthTool

Phase 2 (P2 - Enhanced):
  5. ConfigTool → BriefTool → CtxInspectTool
  6. SkillTool → DiscoverSkillsTool
  7. CronCreateTool → CronDeleteTool → CronListTool
  8. WorkflowTool
  9. SendMessageTool → ListPeersTool → TeamCreateTool → TeamDeleteTool

Phase 3 (P3 - Advanced):
  10. NotebookEditTool → REPLTool → PowerShellTool
  11. LSPTool
  12. Enhanced tools (按依赖排序)
```

## Dependencies

- Task tools: 依赖 `DashMap` 存储
- Scheduling tools: 依赖 `tokio::time`
- MCP tools: 依赖 `crates/mcp` 模块
- Worktree tools: 依赖 `git` 命令可用性
- LSP tools: 依赖 LSP 服务器可用性

## Testing Strategy

每个工具需要 3 个测试用例:
1. **正常流程**: 验证工具正确执行
2. **边界情况**: 超时，空输入，大文件等
3. **错误处理**: 无权限、无效参数等

## Remaining Work

### Enhanced Capabilities (待实现)
| 功能 | 优先级 |
|------|--------|
| Bash 命令历史和自动补全 | P2 |
| Read 语法高亮标记 | P3 |
| Glob 排除模式 | P2 |
| 工具执行结果流式输出 | P3 |
