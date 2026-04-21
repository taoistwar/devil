# Feature Specification: Claude Code Slash Commands Alignment

## Summary

移植 Claude Code 的 100+ 斜杠命令到 devil-agent (Rust)。每个命令对应一个子模块，实现相同功能但使用 Rust 编写。

## Technical Context

- **Language/Version**: Rust 1.75+
- **Primary Dependencies**: tokio, anyhow, serde
- **Storage**: 文件系统配置 (TOML)
- **Testing**: cargo test
- **Target Platform**: Linux/macOS/Windows CLI
- **Project Type**: CLI tool with slash command system

## 命令优先级体系

### 优先级定义

| 优先级 | 说明 | 响应要求 |
|--------|------|----------|
| P1 | 核心功能，无条件执行 | 毫秒级响应，失败需立即反馈 |
| P2 | 重要功能，高频使用 | 秒级响应，允许优雅降级 |
| P3 | 高级功能，低频使用 | 可异步执行，失败可重试 |

### 优先级矩阵

#### P1 - Core Commands (核心命令)

| 命令 | 功能 | 依赖 | 优先级 |
|------|------|------|--------|
| `/help` | 显示帮助信息 | 无 | P1 |
| `/compact` | 手动压缩上下文 | 上下文管理模块 | P1 |
| `/model` | 切换模型 | Provider 配置 | P1 |
| `/clear` | 清除对话 | Session 管理 | P1 |
| `/exit` | 退出 | 无 | P1 |
| `/resume` | 恢复会话 | Session 持久化 | P1 |

#### P2 - Config Commands (配置命令)

| 命令 | 功能 | 依赖 | 优先级 |
|------|------|------|--------|
| `/config` | 配置管理 | Config 模块 | P2 |
| `/login` | 认证登录 | OAuth Provider | P2 |
| `/logout` | 认证登出 | Session 管理 | P2 |
| `/doctor` | 系统诊断 | 工具注册表 | P2 |
| `/cost` | 查看费用 | Usage 统计 | P2 |

#### P3 - Advanced Commands (高级功能)

| 命令 | 功能 | 依赖 | 优先级 |
|------|------|------|--------|
| `/mcp` | MCP 服务器管理 | MCP 协议栈 | P3 |
| `/hooks` | Hook 管理 | Event Bus | P3 |
| `/skills` | 技能管理 | Skills 注册表 | P3 |
| `/tasks` | 任务管理 | Task 调度器 | P3 |
| `/memory` | 记忆管理 | Memory 系统 | P3 |
| `/permissions` | 权限管理 | Permission 模块 | P3 |
| `/agents` | 多代理管理 | Coordinator | P3 |

## 命令分类体系

### 分类结构

```
Commands
├── Core (P1) - 核心命令
├── Config (P2) - 配置命令  
├── Advanced (P3) - 高级功能
├── Edit (P2/P3) - 编辑命令
├── Collaboration (P3) - 协作命令
└── System (P2/P3) - 系统命令
```

### 分类详细定义

#### Core Commands (P1)

核心命令是 Agent 生存所必需的基础命令，提供最基本的人机交互能力。

| 命令 | 功能 | 参数 | 返回结构 |
|------|------|------|----------|
| `/help [command]` | 显示帮助 | `command?: string` | `HelpOutput` |
| `/compact` | 手动压缩上下文 | 无 | `CompactOutput` |
| `/model <name>` | 切换模型 | `name: string` | `ModelOutput` |
| `/clear` | 清除对话 | 无 | `ClearOutput` |
| `/exit` | 退出 | 无 | `ExitOutput` |
| `/resume` | 恢复会话 | 无 | `ResumeOutput` |

#### Config Commands (P2)

配置命令用于管理 Agent 的运行时配置和认证状态。

| 命令 | 功能 | 参数 | 返回结构 |
|------|------|------|----------|
| `/config [key] [value]` | 查看/修改配置 | `key?: string, value?: string` | `ConfigOutput` |
| `/login` | 认证登录 | 无 | `LoginOutput` |
| `/logout` | 认证登出 | 无 | `LogoutOutput` |
| `/doctor` | 系统诊断 | 无 | `DoctorOutput` |
| `/cost` | 查看费用 | 无 | `CostOutput` |

#### Advanced Commands (P3)

高级功能命令提供扩展能力，通常需要较长的执行时间。

| 命令 | 功能 | 参数 | 返回结构 |
|------|------|------|----------|
| `/mcp <subcommand>` | MCP 服务器管理 | `subcommand: enum` | `McpOutput` |
| `/hooks [name]` | Hook 管理 | `name?: string` | `HooksOutput` |
| `/skills [name]` | 技能管理 | `name?: string` | `SkillsOutput` |
| `/tasks [id]` | 任务管理 | `id?: string` | `TasksOutput` |
| `/memory [subcommand]` | 记忆管理 | `subcommand: enum` | `MemoryOutput` |
| `/permissions [action]` | 权限管理 | `action: enum` | `PermissionsOutput` |
| `/agents [command]` | 多代理管理 | `command: enum` | `AgentsOutput` |

#### Edit Commands (P2/P3)

编辑命令用于代码审查和修改协作。

| 命令 | 功能 | 参数 | 优先级 | 依赖 |
|------|------|------|--------|------|
| `/diff [path]` | 查看文件差异 | `path?: string` | P2 | Git 工具 |
| `/review [path]` | 代码审查 | `path?: string` | P3 | Diff 工具 |
| `/plan [description]` | 计划模式 | `description?: string` | P2 | Task 调度 |
| `/vim [subcommand]` | Vim 编辑模式 | `subcommand: enum` | P2 | 编辑器集成 |
| `/fast` | 快速模式 | 无 | P2 | 模型配置 |

#### Collaboration Commands (P3)

协作命令用于多用户协作和消息传递。

| 命令 | 功能 | 参数 | 优先级 | 依赖 |
|------|------|------|--------|------|
| `/share [session]` | 分享对话 | `session?: string` | P3 | Session 管理 |
| `/peers [command]` | 对等连接 | `command: enum` | P3 | P2P 网络 |
| `/send <target> <msg>` | 发送消息 | `target: string, msg: string` | P3 | Peers 连接 |
| `/btw <message>` | 侧注 | `message: string` | P3 | 无 |

#### System Commands (P2/P3)

系统命令用于会话、分支、状态等系统级管理。

| 命令 | 功能 | 参数 | 优先级 | 依赖 |
|------|------|------|--------|------|
| `/branch [name]` | 分支管理 | `name?: string` | P2 | Git 集成 |
| `/session [id]` | 会话管理 | `id?: string` | P2 | Session 持久化 |
| `/status` | 状态查看 | 无 | P2 | 无 |
| `/stats [type]` | 统计信息 | `type?: enum` | P3 | Usage 统计 |
| `/theme [name]` | 主题切换 | `name?: string` | P3 | UI 配置 |

## 命令依赖矩阵

### 依赖类型定义

| 依赖类型 | 符号 | 说明 |
|----------|------|------|
| 强依赖 | `●` | 命令必须依赖，缺失会导致功能完全不可用 |
| 弱依赖 | `○` | 命令可选依赖，缺失时功能降级但仍可用 |
| 内部依赖 | `◐` | 同一分类内的依赖 |

### 依赖矩阵

```
                    │ Tools │ Session │ Config │ Memory │ Permission │ Network │ Git │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
Core Commands       │       │         │        │        │            │         │     │
  /help             │       │         │        │        │            │         │     │
  /compact          │       │   ◐     │        │   ○    │            │         │     │
  /model            │       │         │   ●    │        │            │         │     │
  /clear            │       │   ●     │        │   ○    │            │         │     │
  /exit             │       │   ●    │        │        │            │         │     │
  /resume           │       │   ●    │        │   ○    │            │         │     │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
Config Commands     │       │         │        │        │            │         │     │
  /config           │       │         │   ●    │        │            │         │     │
  /login            │       │   ◐     │        │        │            │   ●    │     │
  /logout           │       │   ●    │        │        │            │   ○    │     │
  /doctor           │   ●   │         │   ○    │        │   ○        │         │     │
  /cost             │       │         │        │        │            │         │     │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
Advanced Commands   │       │         │        │        │            │         │     │
  /mcp              │   ●   │         │   ○    │        │   ○        │   ○    │     │
  /hooks            │   ○   │         │   ●    │        │            │         │     │
  /skills           │       │         │   ○    │   ○    │            │         │     │
  /tasks            │   ○   │   ◐    │        │   ○    │   ○        │         │     │
  /memory           │       │   ◐    │        │   ●    │            │         │     │
  /permissions      │       │         │   ○    │        │   ●        │         │     │
  /agents           │   ◐   │   ◐    │        │   ○    │   ○        │   ○    │     │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
Edit Commands       │       │         │        │        │            │         │     │
  /diff             │   ●   │         │        │        │            │         │  ●  │
  /review           │   ●   │         │        │        │            │         │  ○  │
  /plan             │   ○   │   ◐    │        │   ○    │            │         │     │
  /vim              │   ●   │         │        │        │            │         │     │
  /fast             │       │         │   ●    │        │            │         │     │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
Collaboration       │       │         │        │        │            │         │     │
  /share            │       │   ●    │        │   ○    │            │   ○    │     │
  /peers            │       │   ○    │        │        │   ○        │   ●    │     │
  /send             │       │   ○    │        │        │            │   ●    │     │
  /btw              │       │   ◐    │        │   ○    │            │         │     │
────────────────────┼───────┼─────────┼────────┼────────┼────────────┼─────────┼─────┤
System Commands     │       │         │        │        │            │         │     │
  /branch           │   ●   │         │        │        │            │         │  ●  │
  /session          │       │   ●    │   ○    │   ○    │            │         │     │
  /status           │       │   ◐    │   ○    │        │            │         │     │
  /stats            │       │   ◐    │        │        │            │         │     │
  /theme            │       │         │   ●    │        │            │         │     │
────────────────────┴───────┴─────────┴────────┴────────┴────────────┴─────────┴─────┘
```

### 外部规范依赖

| 规范编号 | 规范名称 | 依赖类型 | 说明 |
|----------|----------|----------|------|
| spec-003 | Tools Alignment | 强依赖 | 所有需要文件/Git/终端操作的命令依赖工具系统 |
| spec-004 | Security Permission Framework | 强依赖 | 所有命令需通过权限校验 |
| spec-006 | Context Alignment | 弱依赖 | `/compact` 等命令依赖上下文管理 |
| spec-007 | Memory System | 弱依赖 | `/memory` 命令依赖记忆系统 |
| spec-008 | Multi-Agent Coordinator | 弱依赖 | `/agents` 命令依赖多代理协调 |

## 统一接口规范

### 错误处理

所有命令必须遵循 `SPEC_DEPENDENCIES.md` 中定义的错误层次结构：

```rust
// 命令执行错误
pub enum CommandError {
    UserInput { code: ErrorCode, message: String },
    Permission { code: ErrorCode, message: String, required_permission: String },
    Resource { code: ErrorCode, message: String, resource_type: String },
    ExternalService { code: ErrorCode, message: String, service: String },
    Internal { code: ErrorCode, message: String },
}
```

### 统一返回结构

```rust
pub struct CommandOutput {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<CommandError>,
    pub execution_time_ms: u64,
}

pub trait SlashCommand: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn priority(&self) -> CommandPriority;
    fn category(&self) -> CommandCategory;
    fn execute(&self, ctx: &CommandContext, args: Value) -> Result<CommandOutput, CommandError>;
    fn dependencies(&self) -> Vec<Dependency>;
}
```

### 日志规范

```rust
// 命令执行日志
log::info!(target: "command", name: self.name(), "Command started");
log::debug!(target: "command", name: self.name(), args = ?args, "Executing command");
log::info!(target: "command", name: self.name(), duration_ms = ms, "Command completed");
log::error!(target: "command", name: self.name(), error = %err, "Command failed");
```

## Functional Requirements

### FR-001: 命令注册系统

- 每个命令是一个独立的 Rust 模块
- 命令通过 `#[derive(SlashCommand)]` 属性注册
- 命令名称与 Claude Code 保持一致

### FR-002: 命令执行框架

- 实现 `SlashCommand` trait
- 支持参数解析 (使用 serde)
- 支持异步执行

### FR-003: 命令分类

- Core Commands: 核心命令 (P1)
- Config Commands: 配置命令 (P2)
- Advanced Commands: 高级功能命令 (P3)
- Edit Commands: 编辑命令 (P2/P3)
- Collaboration Commands: 协作命令 (P3)
- System Commands: 系统命令 (P2/P3)

### FR-004: 优先级机制

- P1 命令优先执行，失败立即返回
- P2 命令可并发执行，失败允许重试
- P3 命令可后台执行，失败进入队列

### FR-005: 依赖检查

- 命令执行前检查依赖模块可用性
- 依赖缺失时提供友好的错误提示
- 支持依赖模块按需加载

### FR-006: 帮助系统

- `/help [command]` 显示指定命令帮助
- `/help` 显示所有命令列表（按分类分组）
- 命令描述与 Claude Code 一致

### FR-007: 命令执行结果

- 返回结构化结果 (JSON)
- 支持成功/失败状态
- 支持错误消息和执行时间

## Success Criteria

1. **命令覆盖率**: 100+ 命令全部实现
2. **功能对齐**: 与 Claude Code 功能一致
3. **优先级支持**: P1/P2/P3 分级明确
4. **分类完整**: 6 大分类无遗漏
5. **依赖清晰**: 依赖矩阵完整准确
6. **编译通过**: cargo build 成功
7. **测试通过**: 单元测试全部通过
8. **CLI 集成**: 与现有 CLI 系统集成

## Architecture

```
src/commands/
├── mod.rs                     # 模块导出
├── traits.rs                   # SlashCommand trait 定义
├── registry.rs                 # 命令注册表
├── priority.rs                  # 优先级管理
├── dependency.rs               # 依赖检查器
│
├── core/                       # Core Commands (P1)
│   ├── help.rs
│   ├── compact.rs
│   ├── model.rs
│   ├── clear.rs
│   ├── exit.rs
│   └── resume.rs
│
├── config/                     # Config Commands (P2)
│   ├── config.rs
│   ├── login.rs
│   ├── logout.rs
│   ├── doctor.rs
│   └── cost.rs
│
├── advanced/                   # Advanced Commands (P3)
│   ├── mcp.rs
│   ├── hooks.rs
│   ├── skills.rs
│   ├── tasks.rs
│   ├── memory.rs
│   ├── permissions.rs
│   └── agents.rs
│
├── edit/                       # Edit Commands (P2/P3)
│   ├── diff.rs
│   ├── review.rs
│   ├── plan.rs
│   ├── vim.rs
│   └── fast.rs
│
├── collaboration/              # Collaboration Commands (P3)
│   ├── share.rs
│   ├── peers.rs
│   ├── send.rs
│   └── btw.rs
│
└── system/                     # System Commands (P2/P3)
    ├── branch.rs
    ├── session.rs
    ├── status.rs
    ├── stats.rs
    └── theme.rs
```

## Dependencies

- 依赖 `specs/003-claude-code-tools-alignment` 的工具系统
- 依赖 `specs/004-security-permission-framework` 的权限系统
- 依赖 `specs/006-context-alignment` 的上下文管理
- 依赖 `specs/007-memory-system` 的记忆系统
- 依赖 `specs/008-multi-agent-coordinator` 的多代理协调
