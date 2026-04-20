# MonkeyCode AI Agent 用户手册

**版本**: 0.1.0 | **更新日期**: 2026-04-20

## 目录

1. [快速开始](#快速开始)
2. [安装配置](#安装配置)
3. [CLI 使用](#cli-使用)
4. [斜杠命令](#斜杠命令)
5. [协调器模式](#协调器模式)
6. [记忆系统](#记忆系统)
7. [工具系统](#工具系统)
8. [故障排除](#故障排除)

---

## 快速开始

### 环境要求

- Rust 1.70+
- Tokio 运行时
- Anthropic API Key（可选，用于真实模型调用）

### 快速安装

```bash
# 克隆项目
git clone https://github.com/taoistwar/devil.git
cd devil

# 构建
cargo build --release

# 运行
export DEVIL_API_KEY="sk-ant-xxx"
./target/release/devil run "帮我分析项目结构"
```

### 快速命令

| 命令 | 说明 |
|------|------|
| `devil run "任务"` | 执行单次任务 |
| `devil repl` | 交互式对话 |
| `devil --help` | 显示帮助 |

---

## 安装配置

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `DEVIL_API_KEY` | API 密钥 | - |
| `DEVIL_MODEL` | 模型名称 | `claude-sonnet-4-20250514` |
| `DEVIL_PROVIDER` | API 提供商 | `anthropic` |
| `DEVIL_BASE_URL` | API 端点 | provider 默认 |
| `DEVIL_MAX_CONTEXT_TOKENS` | 最大上下文 | `200000` |
| `DEVIL_MAX_TURNS` | 最大对话轮次 | `50` |
| `DEVIL_VERBOSE` | 详细日志 | `false` |
| `DEVIL_LOG_JSON` | JSON 日志格式 | `false` |

### 配置文件

位置: `~/.devil/config.toml`

```toml
model = "claude-sonnet-4-20250514"
provider = "anthropic"
api_key = "your-api-key"
base_url = "https://api.anthropic.com"
max_context_tokens = 200000
max_turns = 50
verbose = false
```

---

## CLI 使用

### 基本命令

```bash
# 单次任务执行
devil run "分析 src 目录下的 Rust 文件"

# 交互式 REPL 模式
devil repl

# 显示帮助
devil help

# 显示版本
devil --version

# 显示配置
devil config
```

### 环境变量示例

```bash
# 使用环境变量
DEVIL_API_KEY="sk-ant-xxx" devil run "hello"

# 指定模型
DEVIL_MODEL="claude-opus-4-20250514" devil run "complex task"

# 详细日志
DEVIL_VERBOSE=true devil run "debug task"

# 生产环境 JSON 日志
DEVIL_LOG_JSON=true devil run "task" | jq .
```

### 退出码

| 退出码 | 含义 |
|--------|------|
| 0 | 成功 |
| 1 | 一般错误 |
| 2 | 配置错误 |
| 3 | 缺少 API 密钥 |
| 4 | 无效参数 |
| 5 | 网络错误 |

---

## 斜杠命令

在对话中输入 `/` 触发斜杠命令。

### 常用命令

| 命令 | 说明 | 示例 |
|------|------|------|
| `/help` | 显示帮助 | `/help` |
| `/clear` | 清屏 | `/clear` |
| `/model` | 切换模型 | `/model claude-sonnet` |
| `/continue` | 继续上一步 | `/continue` |
| `/exit` | 退出 | `/exit` |

### 编辑命令

| 命令 | 说明 |
|------|------|
| `/read` | 读取文件 |
| `/write` | 写入文件 |
| `/edit` | 编辑文件 |
| `/diff` | 显示更改 |

### 工具命令

| 命令 | 说明 |
|------|------|
| `/bash` | 执行 Bash 命令 |
| `/grep` | 搜索代码 |
| `/glob` | 文件搜索 |
| `/lsp` | LSP 操作 |

### 高级命令

| 命令 | 说明 |
|------|------|
| `/plan` | 规划任务 |
| `/review` | 代码审查 |
| `/test` | 运行测试 |
| `/memory` | 记忆管理 |
| `/coordinator` | 协调器模式 |
| `/skills` | 技能管理 |
| `/mcp` | MCP 服务器 |

### 命令帮助

```
/help [command]

# 示例
/help memory
/help coordinator
```

---

## 协调器模式

协调器模式允许主 Agent 作为"协调者"并行派发多个 Worker Agent 执行任务。

### 启用方式

```bash
# 设置环境变量
export CLAUDE_CODE_COORDINATOR_MODE=1

# 在 REPL 中启用
/coordinator on

# 查看状态
/coordinator status
```

### 协调器命令

| 命令 | 说明 |
|------|------|
| `/coordinator status` | 查看状态和活跃 Worker |
| `/coordinator on` | 启用协调器模式 |
| `/coordinator off` | 禁用协调器模式 |

### 工作流程

```
用户：修复 auth 模块的 null pointer

协调者：
  1. 并行派发研究任务
     Agent({ description: "调查 auth bug", prompt: "..." })
     Agent({ description: "研究 auth 测试", prompt: "..." })
  
  2. 收到 <task-notification> 结果
  3. 综合发现
  4. 派发实现任务
  5. 派发验证任务
```

### Worker 工具集

**默认模式**: Bash, Read, Edit, Write, Glob, Grep, Todo, MCP, Skill

**Simple 模式**: Bash, Read, Edit

**禁用工具**: TeamCreate, TeamDelete, SendMessage, SyntheticOutput（仅协调者可用）

---

## 记忆系统

记忆系统提供持久化存储和跨会话上下文。

### 记忆命令

```bash
# 列出所有记忆
/memory list

# 按类型列出
/memory list user
/memory list project
/memory list feedback
/memory list reference

# 添加记忆
/memory add user 我的角色 我是后端开发

# 删除记忆
/memory delete user_myrole.md
```

### 记忆类型

| 类型 | 说明 |
|------|------|
| `user` | 用户偏好和设置 |
| `project` | 项目特定信息 |
| `feedback` | 反馈和改进 |
| `reference` | 外部参考 |

### MEMORY.md

项目根目录的 `MEMORY.md` 自动被识别为项目记忆。

---

## 工具系统

### 内置工具

| 工具 | 说明 | 权限级别 |
|------|------|----------|
| `Bash` | 执行 Shell 命令 | 受限 |
| `Read` | 读取文件 | 只读 |
| `Edit` | 编辑文件 | 读写 |
| `Write` | 写入文件 | 读写 |
| `Glob` | 文件搜索 | 只读 |
| `Grep` | 代码搜索 | 只读 |
| `TodoRead` | 读取任务列表 | 只读 |
| `TodoWrite` | 写入任务列表 | 读写 |

### MCP 工具

MCP (Model Context Protocol) 工具通过 MCP 服务器提供。

```bash
# 启动 MCP 服务器
devil mcp start server-name

# 列出可用 MCP 工具
/mcp list

# 查看 MCP 状态
/mcp status
```

---

## 故障排除

### 常见问题

#### API 密钥错误

```
Error: Invalid API key
```

解决方案：检查 `DEVIL_API_KEY` 环境变量是否正确设置。

#### 模型不可用

```
Error: Model not found
```

解决方案：确认 `DEVIL_MODEL` 是有效的模型名称。

#### 工具执行失败

```
Error: Tool execution failed
```

解决方案：
1. 检查工具是否可用 `/help`
2. 查看详细日志 `DEVIL_VERBOSE=true`

#### 上下文过长

```
Error: Context limit exceeded
```

解决方案：
1. 减少 `DEVIL_MAX_CONTEXT_TOKENS`
2. 使用 `/clear` 清屏
3. 手动压缩上下文

### 日志调试

```bash
# 开发环境详细日志
DEVIL_VERBOSE=true devil run "task"

# 生产环境 JSON 日志
DEVIL_LOG_JSON=true devil run "task" | jq .

# 查看结构化日志
RUST_LOG=debug devil run "task"
```

### 性能优化

```bash
# 减少内存占用
DEVIL_MAX_CONTEXT_TOKENS=100000 devil run "task"

# 限制对话轮次
DEVIL_MAX_TURNS=10 devil run "task"
```

---

## 安全最佳实践

### API 密钥管理

```bash
# 推荐：使用环境变量
export DEVIL_API_KEY="sk-ant-xxx"

# 配置文件权限
chmod 600 ~/.devil/config.toml

# 不要提交密钥到版本控制
```

### 安全检查

```bash
# 分析代码安全问题
/bughunter

# 审查代码
/review
```

---

## 配置文件位置

| 配置文件 | 位置 |
|----------|------|
| 主配置 | `~/.devil/config.toml` |
| 记忆目录 | `~/.claude/projects/<project>/memory/` |
| 项目记忆 | `<project>/MEMORY.md` |

---

## 获取帮助

- 查看 README: `devil help`
- 查看命令帮助: `/help <command>`
- 查看架构文档: [架构设计](../.monkeycode/docs/architecture.md)
- 查看开发指南: [快速开始](../.monkeycode/docs/getting-started.md)

---

## 更新日志

### v0.1.0 (2026-04-20)

- 初始版本
- 支持协调器模式
- 支持记忆系统
- 支持 100+ 斜杠命令
- 支持 MCP 工具集成
