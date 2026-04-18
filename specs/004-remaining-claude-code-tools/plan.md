# Implementation Plan: Claude Code Remaining Tools Implementation

**Branch**: `260418-feat-claude-code-tools-alignment` | **Date**: 2026-04-18 | **Spec**: [spec.md](./spec.md)

## Summary

实现 Claude Code 剩余 43 个工具，分 3 个优先级阶段完成。所有工具遵循 Rust-First 标准、Tokio 并发模型和 Claude Code 五要素协议。

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)  
**Primary Dependencies**: tokio, async-trait, serde, reqwest, regex  
**Storage**: 任务状态存储在内存中，定时任务持久化到文件系统  
**Testing**: cargo test, 单元测试 + 集成测试  
**Target Platform**: Linux server environments  
**Project Type**: CLI tool library (devil-agent)  
**Performance Goals**: 工具执行响应 < 100ms（不含实际命令执行）  
**Constraints**: 遵循 Claude Code 五要素协议 (Input, Output, Progress, Permissions, Metadata)  
**Scale/Scope**: 43 个工具，分 3 个阶段实现

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| Rust-First Standards | ✅ PASS | 所有代码使用 idiomatic Rust |
| Tokio Concurrency Model | ✅ PASS | 使用 `#[async_trait]` 和 Tokio runtime |
| Claude Code Reference Parity | ✅ PASS | 工具名称、参数与参考实现一致 |
| Robust Error Handling | ✅ PASS | 使用 `anyhow::Result<T>` 和 `thiserror` |
| Tool-First Architecture | ✅ PASS | 所有功能通过工具暴露 |

## Phase 1: Core Infrastructure (P1)

### 1.1 Task Management Tools (5 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| TaskCreateTool | 创建任务 | `TaskCreateTool/TaskCreateTool.ts` |
| TaskUpdateTool | 更新任务状态 | `TaskUpdateTool/TaskUpdateTool.ts` |
| TaskListTool | 列出所有任务 | `TaskListTool/TaskListTool.ts` |
| TaskGetTool | 获取任务详情 | `TaskGetTool/TaskGetTool.ts` |
| TaskStopTool | 停止任务 | `TaskStopTool/TaskStopTool.ts` |

**实现要点**:
- 任务状态机: `pending` → `in_progress` → `completed` / `failed` / `stopped`
- 使用 `DashMap` 存储任务状态
- 任务 ID 使用 UUID

### 1.2 Planning Mode Tools (2 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| EnterPlanModeTool | 进入规划模式 | `EnterPlanModeTool/EnterPlanModeTool.ts` |
| ExitPlanModeTool | 退出规划模式 | `ExitPlanModeTool/ExitPlanModeV2Tool.ts` |

**实现要点**:
- Agent 状态机增加 `planning` 状态
- 规划模式使用不同的 system prompt

### 1.3 Worktree Tools (2 tools)

| 工具 | 说明 | 参考文件 |
|------|------|----------|
| EnterWorktreeTool | 进入/创建 worktree | `EnterWorktreeTool/EnterWorktreeTool.ts` |
| ExitWorktreeTool | 退出 worktree | `ExitWorktreeTool/ExitWorktreeTool.ts` |

**实现要点**:
- 使用 `git worktree` 命令
- 跟踪多个工作目录状态

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

## Phase 2: Enhanced Tools (P2)

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

## Phase 3: Advanced Tools (P3)

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

### 3.3 Enhanced Tools (13 tools)

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
| MonitorTool | 监控 | (空目录) |
| SleepTool | 延迟执行 | `SleepTool/SleepTool.ts` |
| ToolSearchTool | 搜索工具 | `ToolSearchTool/ToolSearchTool.ts` |
| RemoteTriggerTool | 远程触发 | `RemoteTriggerTool/RemoteTriggerTool.ts` |

## Project Structure

```text
crates/agent/src/
├── tools/
│   ├── builtin.rs              # 10 个已完成工具
│   ├── builtin_extended.rs      # Phase 1 新增工具 (Task, Planning, Worktree)
│   ├── builtin_mcp.rs          # Phase 1 MCP 工具
│   ├── builtin_config.rs       # Phase 2 Config/Skills 工具
│   ├── builtin_scheduling.rs    # Phase 2 Scheduling/Workflow 工具
│   ├── builtin_comm.rs          # Phase 2 Communication 工具
│   ├── builtin_file.rs          # Phase 3 File 工具
│   ├── builtin_lsp.rs           # Phase 3 LSP 工具
│   ├── builtin_enhanced.rs      # Phase 3 Enhanced 工具
│   ├── tool.rs                  # Tool trait 定义
│   ├── registry.rs               # 工具注册表
│   ├── executor.rs               # 工具执行器
│   └── build_tool.rs             # 工具构建器
├── subagent/
│   └── mod.rs                   # 子代理模块
├── task/
│   ├── mod.rs                   # 任务管理模块
│   ├── store.rs                 # 任务存储
│   └── scheduler.rs             # 定时调度器
├── mcp/
│   └── mod.rs                   # MCP 集成
└── core.rs                      # Agent 主模块
```

## Implementation Order

```
Phase 1 (P1):
  1. TaskCreateTool → TaskUpdateTool → TaskListTool → TaskGetTool → TaskStopTool
  2. EnterPlanModeTool → ExitPlanModeTool
  3. EnterWorktreeTool → ExitWorktreeTool
  4. MCPTool → ListMcpResourcesTool → ReadMcpResourceTool → McpAuthTool

Phase 2 (P2):
  5. ConfigTool → BriefTool → CtxInspectTool
  6. SkillTool → DiscoverSkillsTool
  7. CronCreateTool → CronDeleteTool → CronListTool
  8. WorkflowTool
  9. SendMessageTool → ListPeersTool → TeamCreateTool → TeamDeleteTool

Phase 3 (P3):
  10. NotebookEditTool → REPLTool → PowerShellTool
  11. LSPTool
  12. Enhanced tools (按依赖排序)
```

## Complexity Tracking

> 无复杂度违规

## Dependencies

- Task tools: 依赖 `DashMap` 存储
- Scheduling tools: 依赖 `tokio::time`
- MCP tools: 依赖 `crates/mcp` 模块
- Worktree tools: 依赖 `git` 命令可用性
- LSP tools: 依赖 LSP 服务器可用性

## Testing Strategy

每个工具需要 3 个测试用例:
1. **正常流程**: 验证工具正确执行
2. **边界情况**: 超时、空输入、大文件等
3. **错误处理**: 无权限、无效参数等
