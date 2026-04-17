# 钩子系统设计文档

## 概述

钩子系统是 Agent 的"神经系统"，遵循**观察者模式 + 责任链模式**。权限管线决定 Agent"能否"执行操作，钩子系统决定执行"前后"发生什么。

### 设计哲学

1. **关注点分离**：权限系统负责安全准入，钩子系统负责扩展行为
2. **非侵入式**：钩子执行不影响核心 Agent 逻辑
3. **可组合性**：多个钩子可以链式执行，共同决策
4. **安全优先**：三层门禁防止恶意钩子执行

## 架构设计

```
┌──────────────────────────────────────────────────────────────┐
│                      Agent Core Loop                          │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                    Hook Event Dispatcher                      │
│  • PreToolUse / PostToolUse / PostToolUseFailure             │
│  • UserPromptSubmit / Stop / StopFailure                      │
│  • PermissionRequest / PermissionDenied                       │
│  • SessionStart / SessionEnd / Setup                          │
│  • SubagentStart / SubagentStop                               │
│  • PreCompact / PostCompact                                   │
│  • Notification / ConfigChange / CwdChanged / FileChanged     │
│  • Elicitation / ElicitationResult                            │
│  • TaskCreated / TaskCompleted / TeammateIdle                 │
│  • InstructionsLoaded / WorktreeCreate / WorktreeRemove       │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                  HookRegistry.getMatchingHooks()              │
│  • 多来源收集：user > project > local > plugin > builtin      │
│  • 匹配器过滤：精确匹配 | 管道分隔 | 正则                      │
│  • if 条件过滤：权限规则语法                                  │
│  • 优先级排序：userSettings > project > local > plugin...    │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│               HookSecurityGuard.check()                       │
│  ┌───────────────────────────────────────────────────────┐   │
│  │ 第一层：disableAllHooks = true → 拒绝所有              │   │
│  └───────────────────────────────────────────────────────┘   │
│                              │                                │
│                              ▼                                │
│  ┌───────────────────────────────────────────────────────┐   │
│  │ 第二层：allowManagedHooksOnly = true → 仅内置/托管     │   │
│  └───────────────────────────────────────────────────────┘   │
│                              │                                │
│                              ▼                                │
│  ┌───────────────────────────────────────────────────────┐   │
│  │ 第三层：工作区信任检查 → 未信任则要求确认              │   │
│  └───────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                    HookExecutor.execute()                     │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │  Command    │  │   Prompt    │  │    Agent    │          │
│  │  (Shell)    │  │   (LLM)     │  │  (Subagent) │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│  │    HTTP     │  │  Callback   │  │  Function   │          │
│  │  (Request)  │  │   (JS fn)   │  │  (Runtime)  │          │
│  └─────────────┘  └─────────────┘  └─────────────┘          │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│                  HookResponse Protocol                        │
│  {                                                            │
│    "continue": true/false,                                    │
│    "suppressOutput": true/false,                              │
│    "stopReason": "原因",                                      │
│    "decision": "approve" | "block",                           │
│    "systemMessage": "注入上下文的消息",                        │
│    "hookSpecificOutput": {                                    │
│      "permissionDecision": "allow" | "deny" | "ask",          │
│      "updatedInput": {...},                                   │
│      "additionalContext": "额外上下文"                         │
│    }                                                          │
│  }                                                            │
└──────────────────────────────────────────────────────────────┘
```

## 钩子类型（6 种）

### Command 钩子

**执行方式**: Shell 命令（bash/PowerShell）

**适用场景**:
- Git 提交前检查
- 代码格式化验证
- 安全扫描脚本
- CI/CD 集成

**配置示例**:
```json
{
  "type": "command",
  "command": "npm run lint && npm run test",
  "if": "PreToolUse(Bash(git commit*))",
  "shell": "bash",
  "timeout": 300,
  "statusMessage": "运行代码检查...",
  "once": false,
  "async": false,
  "asyncRewake": false
}
```

### Prompt 钩子

**执行方式**: LLM 提示词评估

**适用场景**:
- 代码规范提醒
- 安全意识检查
- 最佳实践建议

**配置示例**:
```json
{
  "type": "prompt",
  "prompt": "检查用户的代码修改是否符合项目规范，重点关注：1. 命名约定 2. 错误处理 3. 日志记录",
  "if": "PostToolUse(Write)",
  "timeout": 30,
  "model": "claude-haiku",
  "statusMessage": "检查代码规范...",
  "once": false
}
```

### Agent 钩子

**执行方式**: 启动子代理执行验证

**适用场景**:
- 复杂测试验证
- 代码审查
- 安全性分析

**配置示例**:
```json
{
  "type": "agent",
  "prompt": "验证修改后的代码是否通过所有单元测试，如果失败请分析原因",
  "if": "PostToolUse(Write)",
  "timeout": 300,
  "model": "claude-sonnet-4",
  "statusMessage": "运行测试验证...",
  "once": false
}
```

### HTTP 钩子

**执行方式**: HTTP POST 请求

**适用场景**:
- 远程 CI 服务集成
- Webhook 通知
- 外部审计服务

**配置示例**:
```json
{
  "type": "http",
  "url": "https://ci.example.com/api/hooks/pre-commit",
  "if": "PreToolUse(Bash(git commit*))",
  "timeout": 60,
  "headers": {
    "Authorization": "Bearer $CI_TOKEN",
    "Content-Type": "application/json"
  },
  "allowedEnvVars": ["CI_TOKEN"],
  "statusMessage": "通知 CI 服务...",
  "once": false
}
```

### Callback 钩子

**执行方式**: 运行时注册的 JS 回调函数

**适用场景**:
- SDK 内部钩子
- 插件系统钩子

**注意**: 仅运行时存在，不可持久化

### Function 钩子

**执行方式**: 运行时动态注册的函数

**适用场景**:
- Agent 前置注册钩子
- Skill 系统钩子

**注意**: 仅运行时存在，不可持久化

## 生命周期事件（26 种）

### 会话管理

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `SessionStart` | 会话启动 | `source` |
| `SessionEnd` | 会话结束 | `reason` |
| `Setup` | 初始化完成 | `trigger` |

### 用户交互

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `UserPromptSubmit` | 用户提交消息 | - |
| `Stop` | Agent 停止响应 | - |
| `StopFailure` | Agent 停止失败 | `error` |

### 工具执行

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `PreToolUse` | 工具调用前 | `tool_name` |
| `PostToolUse` | 工具调用后（成功） | `tool_name` |
| `PostToolUseFailure` | 工具调用后（失败） | `tool_name` |

### 权限管理

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `PermissionRequest` | 权限请求 | `tool_name` |
| `PermissionDenied` | 权限被拒 | `tool_name` |

### 子代理

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `SubagentStart` | 子代理启动 | `agent_type` |
| `SubagentStop` | 子代理停止 | `agent_type` |

### 上下文压缩

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `PreCompact` | 压缩前 | `trigger` |
| `PostCompact` | 压缩后 | `trigger` |

### 协作

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `TeammateIdle` | Teammate 空闲 | - |
| `TaskCreated` | 任务创建 | - |
| `TaskCompleted` | 任务完成 | - |

### MCP

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `Elicitation` | MCP 服务器请求用户输入 | `mcp_server_name` |
| `ElicitationResult` | Elicitation 结果返回 | `mcp_server_name` |

### 通知

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `Notification` | 系统通知事件 | `notification_type` |

### 环境

| 事件 | 触发时机 | 匹配字段 |
|------|---------|---------|
| `ConfigChange` | 配置变更 | `source` |
| `CwdChanged` | 工作目录变更 | - |
| `FileChanged` | 文件变更 | `file_path` |
| `InstructionsLoaded` | 指令加载 | `load_reason` |
| `WorktreeCreate` | Worktree 创建 | - |
| `WorktreeRemove` | Worktree 移除 | - |

## 安全门禁（三层）

### 第一层：全局禁用

**配置**:
```json
{
  "hooks": {
    "disableAllHooks": true
  }
}
```

**效果**: 所有钩子被禁用，无论来源和类型

### 第二层：仅托管钩子

**配置**:
```json
{
  "hooks": {
    "allowManagedHooksOnly": true
  }
}
```

**效果**: 仅允许内置钩子和受信任插件的钩子

### 第三层：工作区信任检查

**机制**: 
- 首次进入未信任工作区时弹出确认对话框
- 用户确认后，工作区路径加入信任列表
- 未信任工作区的钩子执行被阻止

**例外**: 非交互模式下隐式信任

## 钩子响应协议

### 标准 JSON Schema

```json
{
  "continue": true,
  "suppressOutput": false,
  "stopReason": null,
  "decision": "approve",
  "reason": "检查通过",
  "systemMessage": null,
  "hookSpecificOutput": {
    "hookEventName": "PreToolUse",
    "permissionDecision": "allow",
    "permissionDecisionReason": "匹配安全规则",
    "updatedInput": null,
    "additionalContext": "建议添加日志记录"
  }
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|------|------|------|
| `continue` | boolean | 是否继续执行（默认 `true`） |
| `suppressOutput` | boolean | 抑制 stdout 输出（默认 `false`） |
| `stopReason` | string | 阻止执行的原因 |
| `decision` | "approve" \| "block" | 全局决策 |
| `reason` | string | 决策原因 |
| `systemMessage` | string | 注入到上下文的系统消息 |

### hookSpecificOutput 按事件类型

| 事件 | 专有字段 | 作用 |
|------|---------|------|
| `PreToolUse` | `permissionDecision`, `permissionDecisionReason`, `updatedInput`, `additionalContext` | 拦截/修改工具输入 |
| `PostToolUse` | `additionalContext`, `updatedMCPToolOutput` | 修改 MCP 工具输出 |
| `PostToolUseFailure` | `additionalContext` | 失败后注入上下文 |
| `UserPromptSubmit` | `additionalContext` | 注入额外上下文 |
| `SessionStart` | `additionalContext`, `initialUserMessage`, `watchPaths` | 设置初始消息和文件监控 |
| `PermissionRequest` | `decision` | 权限请求的 Hook 决策 |
| `PermissionDenied` | `retry` | 指示是否重试 |
| `Elicitation` | `action`, `content` | 控制用户输入对话框 |

## 匹配机制

### matcher 模式

| 模式 | 示例 | 说明 |
|------|------|------|
| 精确匹配 | `"Write"` | 仅匹配 Write 工具 |
| 管道分隔 | `"Write\|Read\|Edit"` | 匹配多个工具名 |
| 正则匹配 | `"^Bash(git.*)"` | 匹配 git 开头的 Bash 命令 |
| 通配符 | `"*"` 或 `""` | 匹配所有事件 |

### if 条件过滤

使用权限规则语法过滤钩子执行时机：

```json
{
  "hooks": [{
    "command": "check-git-branch.sh",
    "if": "Bash(git push*)"
  }]
}
```

**解析规则**:
- `ToolName(pattern)` - 工具名 + 参数模式
- `*` - 通配符
- 支持 tree-sitter AST 级别解析（Bash 工具）

## 执行模式

### 同步模式（默认）

```json
{
  "command": "npm test",
  "async": false
}
```

- 阻塞等待执行完成
- 最长超时时间：`timeout` 秒（默认 600 秒）

### 异步模式

```json
{
  "command": "npm run build",
  "async": true
}
```

- 后台执行，不阻塞主流程
- stdout 第一行输出：`{"async":true}`
- 通过 `AsyncHookRegistry` 管理

### 异步唤醒模式

```json
{
  "command": "long-running-check.sh",
  "asyncRewake": true
}
```

- 后台执行
- 退出码 **2** 时唤醒模型
- 注入 `task-notification` 或 `queued_command`

## 优先级排序

**从高到低**:

1. User Settings（用户设置）
2. Project Settings（项目设置）
3. Local Settings（本地设置）
4. Plugin Hooks（插件钩子）
5. Builtin Hooks（内置钩子）
6. Session Hooks（会话钩子）

**冲突解决**: 高优先级钩子的决策优先生效

## 代码结构

```
crates/agent/src/hooks/
├── mod.rs           # 模块入口
├── types.rs         # 钩子类型定义（6 种类型）
├── events.rs        # 26 个生命周期事件
├── response.rs      # 钩子响应协议
├── matcher.rs       # 匹配器实现
├── registry.rs      # 钩子注册表
├── security.rs      # 安全门禁
├── executor.rs      # 钩子执行引擎
└── builtin/
    └── mod.rs       # 内置钩子
```

## 集成点

### 与权限系统集成

```rust
// 在 PermissionRequest 钩子中修改权限决策
let response = HookResponse::ok()
    .with_permission_decision(
        PermissionDecision::Allow,
        "匹配了可信命令模式"
    );
```

### 与设置系统集成

```rust
// 从设置中加载钩子配置
let config = settings.hooks.clone();
let guard = HookSecurityGuard::new(config, workspace_path);
```

### 与上下文管理系统集成

```rust
// 在 hookSpecificOutput 中注入额外上下文
let response = HookResponse::ok()
    .with_context("建议使用 Result 类型包装返回值");
```

## 最佳实践

1. **最小权限**: 钩子命令使用最小必要权限
2. **超时设置**: 始终设置合理的 timeout 值
3. **错误处理**: 钩子失败不阻断主流程（除非安全原因）
4. **日志记录**: 使用 `suppressOutput: false` 记录调试信息
5. **异步长任务**: 耗时操作使用 `async: true` 模式

## 参考实现

- Claude Code: `src/utils/hooks.ts` (5177 行)
- Hook Schema: `src/schemas/hooks.ts`
- Hook Events: `src/entrypoints/agentSdkTypes.ts`

## 待办事项

- [ ] 实现完整的 Prompt 钩子 LLM 调用
- [ ] 实现异.
- [ ] 集成到 Agent 主循环
- [ ] 编写单元测试
- [ ] 编写集成测试

