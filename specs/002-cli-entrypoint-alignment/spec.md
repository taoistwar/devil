# Feature Specification: CLI Entry Point Alignment

## Metadata

| Field | Value |
|-------|-------|
| **Spec ID** | 002 |
| **Feature Branch** | `260417-feat-cli-entrypoint` |
| **Created** | 2026-04-17 |
| **Last Updated** | 2026-04-21 |
| **Status** | Draft |
| **Priority** | P0 |
| **Dependencies** | 001 |
| **Dependents** | 005 |

---

## 1. Concept & Vision

### 1.1 Summary

CLI 是用户与 Devil Agent 交互的主要方式。规范定义 CLI 的入口点、命令路由、配置管理和生命周期。

### 1.2 设计原则

1. **Fast Path**: 版本信息和帮助应立即返回，不加载模块
2. **一致性**: 所有命令遵循统一的输出格式
3. **可扩展性**: 新命令通过注册表添加，不修改核心代码

---

## 2. Command Structure

### 2.1 Command Hierarchy

```
devil
├── [global options]
├── run <prompt>          # 单任务执行
├── repl                   # 交互式 REPL
├── web                    # Web 服务模式
├── config                 # 配置管理
├── --version, -v, -V     # 版本信息
├── --help, -h             # 帮助信息
└── <subcommand>           # 子命令
```

### 2.2 Global Options

| Option | 说明 | 示例 |
|--------|------|------|
| `--version`, `-v`, `-V` | 显示版本 | `devil -v` |
| `--help`, `-h` | 显示帮助 | `devil --help` |
| `--config <path>` | 指定配置文件 | `devil --config /path/to/config.toml` |
| `--dry-run` | 模拟执行 | `devil --dry-run run "task"` |
| `--verbose` | 详细输出 | `devil --verbose run "task"` |
| `--quiet`, `-q` | 静默模式 | `devil -q run "task"` |

---

## 3. User Scenarios

### US-001: 版本查询 (Fast Path)

**Priority**: P1

```
Scenario: 用户查询版本
  Given 用户运行 "devil --version"
  When 命令执行
  Then 在 100ms 内返回版本信息
  And 不加载任何模块
  And 进程正常退出 (exit code 0)
```

**实现要求**:
- 版本信息编译时静态嵌入
- 不使用 anyhow, tokio 等运行时依赖

### US-002: 帮助信息

**Priority**: P1

```
Scenario: 显示帮助
  Given 用户运行 "devil --help"
  Then 显示所有可用命令
  And 显示全局选项说明
  And 显示示例
```

**帮助格式**:
```
Devil AI Agent v0.1.0

USAGE:
  devil [OPTIONS] <COMMAND>

COMMANDS:
  run <PROMPT>    Execute a single task
  repl            Start interactive REPL
  web             Start web server
  config          Configuration management
  help            Show this help

OPTIONS:
  -v, --version   Show version
  -h, --help      Show help
  --config PATH   Config file path

Run 'devil help <COMMAND>' for more information.
```

### US-003: 单任务执行

**Priority**: P1

```
Scenario: 执行单任务
  Given 用户运行 "devil run <prompt>"
  When 任务完成
  Then 显示结果
  And 进程正常退出
```

**错误处理**:
```
# 缺少参数
devil run
Error: Missing required argument 'prompt'
Usage: devil run <PROMPT>

# 任务失败
devil run "invalid task"
Error: Task execution failed: <reason>
```

### US-004: 交互式 REPL

**Priority**: P1

```
Scenario: 启动 REPL
  Given 用户运行 "devil repl"
  Then 显示欢迎信息
  And 显示提示符 "devil> "
  And 等待用户输入
```

**REPL 命令**:
```
devil> help           # 显示帮助
devil> /exit          # 退出 REPL
devil> /clear         # 清除历史
devil> <text>         # 发送消息
```

### US-005: Web 服务模式

**Priority**: P2

```
Scenario: 启动 Web 服务
  Given 用户运行 "devil web"
  Then 启动 HTTP 服务器
  And 显示服务地址
  And 保持前台运行
```

---

## 4. Dispatcher Architecture

### 4.1 Command Registry Pattern

```rust
trait Command {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, ctx: &CliContext, args: &[String]) -> Result<()>;
}

struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn register<C: Command + 'static>(&mut self, command: C) {
        self.commands.insert(command.name().to_string(), Box::new(command));
    }
    
    pub fn dispatch(&self, ctx: &CliContext, input: &[String]) -> Result<()> {
        match input.first() {
            Some(cmd) => self.commands.get(cmd)
                .ok_or_else(|| anyhow!("Unknown command: {}", cmd))?
                .execute(ctx, &input[1..]),
            None => self.commands.get("help").unwrap().execute(ctx, &[]),
        }
    }
}
```

### 4.2 Command Execution Flow

```
CLI Input: "devil run 'analyze code'"
    │
    ▼
┌─────────────────────────────────────┐
│ Parse global options               │
│ --verbose, --config, etc.          │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Fast path check                    │
│ -v/--version → print and exit      │
│ -h/--help → show help and exit     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Command Dispatch                    │
│ "run" → RunCommand::execute()       │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Session Creation                   │
│ - Load config                      │
│ - Initialize tools                  │
│ - Load memory                       │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Agent Execution                    │
│ - Run task                         │
│ - Stream output                     │
└─────────────────────────────────────┘
    │
    ▼
┌─────────────────────────────────────┐
│ Cleanup & Exit                     │
└─────────────────────────────────────┘
```

---

## 5. Configuration Management

### 5.1 Config Precedence

```
Highest → Lowest:
1. Environment variables (DEVIL_*)
2. Command line arguments
3. Config file (~/.devil/config.toml)
4. Default values
```

### 5.2 Config File Schema

```toml
[agent]
name = "devil"
model = "claude-sonnet-4-20250514"
max_turns = 100
max_context_tokens = 200000

[cli]
color = true
verbose = false
prompt = "devil> "

[tools]
timeout_seconds = 300
allow_destructive = false
parallel_execution = true

[permissions]
mode = "ask"  # ask | auto | bypass
rules_file = "~/.devil/rules.toml"

[logging]
level = "info"  # trace | debug | info | warn | error
format = "pretty"  # pretty | json
file = "~/.devil/logs/devil.log"

[web]
host = "127.0.0.1"
port = 8080
api_key = ""  # If set, requires API key for requests
```

### 5.3 Environment Variables

| Variable | Type | 说明 |
|----------|------|------|
| `DEVIL_API_KEY` | String | API 密钥 |
| `DEVIL_BASE_URL` | String | API 基础 URL |
| `DEVIL_CONFIG` | String | 配置文件路径 |
| `DEVIL_MODEL` | String | 默认模型 |
| `DEVIL_VERBOSE` | Bool | 详细输出 |
| `DEVIL_MOCK_MODEL` | Bool | 使用模拟模式 |

---

## 6. Error Handling

### 6.1 Exit Codes

| Code | Meaning |
|------|---------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 解析错误 |
| 3 | 配置错误 |
| 4 | 执行错误 |
| 5 | 权限错误 |

### 6.2 Error Output Format

```
Error: <message>
  Caused by: <cause>
  Context: <context>

Run 'devil --help' for usage information.
```

---

## 7. Signal Handling

### 7.1 Graceful Shutdown

```
SIGINT (Ctrl+C):
1. 停止接收新输入
2. 完成当前操作
3. 保存状态
4. 清理资源
5. 退出 (exit code 0)

SIGTERM:
1. 停止接收新输入
2. 等待当前操作完成 (最多 5s)
3. 强制终止
4. 退出 (exit code 1)
```

---

## 8. Dependencies

| Spec | Type | Purpose |
|------|------|---------|
| 001 | Strong | CLI 依赖 Agent 核心 |
| 005 | Strong | 斜杠命令通过 CLI 调度 |

---

## 9. Acceptance Criteria

| ID | Criteria | Test Method |
|----|----------|-------------|
| AC-001 | `--version` 在 100ms 内返回 | Benchmark |
| AC-002 | 所有命令有集成测试 | `cargo test` |
| AC-003 | 帮助信息完整且准确 | 人工评审 |
| AC-004 | 未知命令返回 exit code 2 | 单元测试 |
| AC-005 | 配置优先级正确 | 单元测试 |
