# Feature Specification: Terminal AI Coding Agent

## Metadata

| Field | Value |
|-------|-------|
| **Spec ID** | 001 |
| **Feature Branch** | `260417-feat-terminal-ai-coding-agent` |
| **Created** | 2026-04-17 |
| **Last Updated** | 2026-04-21 |
| **Status** | Draft |
| **Priority** | P0 (Foundation) |
| **Dependencies** | None (base spec) |
| **Dependents** | 002, 003, 004, 005, 006, 007, 008, 009 |

---

## 1. Concept & Vision

### 1.1 Summary

构建一个终端 AI 编码助手，类似于 Claude Code。它能够：

- 理解代码库结构并进行推理
- 执行 shell 命令
- 读取和写入文件
- 与用户进行交互式对话来完成编程任务

### 1.2 设计原则

1. **Fail-Closed**: 安全性相关的默认配置拒绝操作
2. **最小惊讶原则**: Agent 行为应该符合用户预期
3. **可恢复性**: 错误应该可追溯、可恢复
4. **透明性**: 用户应该始终了解 Agent 在做什么

### 1.3 核心价值

| 价值 | 说明 |
|------|------|
| 效率 | 处理耗时的重复性编码任务 |
| 准确性 | 理解代码上下文，避免盲目修改 |
| 协作性 | 用户全程参与，可以纠正和引导 |
| 安全性 | 多层权限检查，防止意外破坏 |

---

## 2. User Scenarios & Testing

### 2.1 User Story Matrix

| ID | Story | Priority | Complexity |
|----|-------|----------|------------|
| US-001 | 交互式任务解决 | P1 | High |
| US-002 | 代码库探索和推理 | P1 | Medium |
| US-003 | 安全文件操作 | P1 | High |
| US-004 | Shell 命令执行 | P1 | Medium |
| US-005 | 用户反馈循环 | P2 | Low |

### 2.2 Detailed User Stories

#### US-001: 交互式任务解决

**描述**: 开发者在项目目录启动终端助手，提供编程任务，助手分析代码库、创建计划、执行修改，并在整个过程中保持用户知情。

**验收标准** (Gherkin):

```
Scenario: 成功完成代码修改任务
  Given 用户在项目目录运行 devil
  And 用户提供任务 "添加用户认证到登录端点"
  When 助手开始处理任务
  Then 助手确认理解任务
  And 助手分析代码结构
  And 助手创建修改计划
  And 助手展示计划等待用户确认
  And 用户确认计划
  And 助手执行修改
  And 修改成功完成

Scenario: 用户中途纠正助手
  Given 助手正在执行任务
  When 用户说 "等一下，应该先检查权限"
  Then 助手暂停当前操作
  And 助手感谢用户纠正
  And 助手调整计划
```

**测试用例**:
- TC-001-01: 助手成功完成一个简单的代码修改任务
- TC-001-02: 助手在用户纠正后调整计划
- TC-001-03: 助手在任务完成后提供总结

#### US-002: 代码库探索

**描述**: 助手能够智能探索不熟悉的代码库，理解其结构、模式和关键组件。

**验收标准**:

```
Scenario: 分析项目结构
  Given 用户要求助手理解项目
  When 助手探索代码库
  Then 助手识别关键目录、文件和架构模式
  And 助手报告入口点和依赖关系

Scenario: 查找特定功能
  Given 用户需要找到特定功能
  When 助手搜索代码库
  Then 助手报告相关文件位置和代码片段
```

**测试用例**:
- TC-002-01: 助手识别 Rust 项目的模块结构
- TC-002-02: 助手找到特定函数定义
- TC-002-03: 助手理解组件间的依赖关系

#### US-003: 安全文件操作

**描述**: 助手可以读取现有文件并写入新内容，同时确保变更安全应用并有适当备份。

**验收标准**:

```
Scenario: 读取大文件
  Given 助手需要读取文件
  When 文件超过 10000 行
  Then 助手分块读取
  And 助手报告文件大小警告

Scenario: 原子写入
  Given 助手需要写入文件
  When 用户批准修改
  Then 助手先写入临时文件
  And 验证内容后原子替换原文件
  And 保留原文件备份

Scenario: 写入失败恢复
  Given 写入过程中发生错误
  Then 原文件保持不变
  And 助手报告错误详情
```

**测试用例**:
- TC-003-01: 助手正确读取各种编码的文件
- TC-003-02: 助手正确写入 UTF-8 文件
- TC-003-03: 写入失败时原文件不变

#### US-004: Shell 命令执行

**描述**: 助手执行 shell 命令来运行测试、构建系统、linting 等开发任务。

**验收标准**:

```
Scenario: 执行构建命令
  Given 助手需要验证代码
  When 助手执行 "cargo build"
  Then 助手捕获输出
  And 助手报告构建结果

Scenario: 命令超时处理
  Given 助手执行长时间命令
  When 命令超过 5 分钟
  Then 助手终止命令
  And 助手报告超时

Scenario: 危险命令确认
  Given 助手需要执行 rm -rf
  Then 助手显示危险警告
  And 助手等待用户明确确认
```

**测试用例**:
- TC-004-01: 助手成功执行 cargo test
- TC-004-02: 超时命令被正确终止
- TC-004-03: rm 命令需要用户确认

#### US-005: 用户反馈循环

**描述**: 助手和开发者可以持续对话，开发者实时引导、纠正或重定向助手的工作。

**验收标准**:

```
Scenario: 中途改变方向
  Given 助手正在执行任务
  When 用户提供额外指令
  Then 助手确认并调整方法

Scenario: 助手误解时纠正
  Given 助手做出错误假设
  When 用户纠正
  Then 助手相应修订计划
```

**测试用例**:
- TC-005-01: 用户指令被正确处理
- TC-005-02: 助手正确响应纠正

---

## 3. Technical Specification

### 3.1 Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         CLI Entry                               │
│                    (devil run / devil repl)                     │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Session Manager                             │
│  • 创建/恢复会话                                                 │
│  • 加载配置和记忆                                                │
│  • 管理会话状态                                                  │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Agent Core                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   State    │  │   Loop     │  │  Context    │             │
│  │  Machine   │  │            │  │  Manager    │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Tool System                                  │
│  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐     │
│  │  Read  │ │ Write  │ │  Edit  │ │  Bash  │ │  Glob  │     │
│  └────────┘ └────────┘ └────────┘ └────────┘ └────────┘     │
│                           + Permissions Layer (Spec-004)        │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                     LLM Provider                                │
│  • OpenAI / Anthropic / Azure                                  │
│  • Streaming responses                                          │
│  • Error handling & retry                                      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Session Lifecycle

```
Session States:
                                                         
  ┌─────────┐    start()    ┌───────────┐    task_done    ┌──────────┐
  │ Created │──────────────►│ Running   │────────────────►│ Finished │
  └─────────┘               └───────────┘                 └──────────┘
       │                         │                             
       │                         │ pause()                       
       │                         ▼                             
       │                    ┌──────────┐                        
       └───────────────────►│ Paused   │                        
         resume()           └──────────┘                        
                                │                               
                                │ stop()                        
                                ▼                               
                           ┌──────────┐                        
                           │ Stopped  │                        
                           └──────────┘                        
```

### 3.3 Context Flow

```
User Input
    │
    ▼
┌─────────────────┐
│ Input Validator │ ←── Permission Check
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Context Builder │ ←── Memory, Git Status, Config
└─────────────────┘
    │
    ▼
┌─────────────────┐
│  LLM Provider   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Tool Executor   │
└─────────────────┘
    │
    ▼
┌─────────────────┐
│ Response to    │
│ User            │
└─────────────────┘
```

---

## 4. Data Models

### 4.1 Core Entities

```rust
/// 会话
struct Session {
    id: Uuid,
    status: SessionStatus,
    config: SessionConfig,
    created_at: DateTime<Utc>,
    last_activity: DateTime<Utc>,
}

/// 用户消息
struct UserMessage {
    content: Vec<ContentBlock>,
    metadata: MessageMetadata,
}

/// 助手消息
struct AssistantMessage {
    content: Vec<ContentBlock>,
    tool_use: Vec<ToolUseBlock>,
    metadata: MessageMetadata,
}

/// 工具调用结果
struct ToolResult {
    tool: String,
    input: serde_json::Value,
    output: ToolOutput,
    duration_ms: u64,
}
```

### 4.2 Enums

```rust
enum SessionStatus {
    Created,
    Running,
    Paused,
    Stopped,
    Finished,
}

enum ContentBlock {
    Text(String),
    Image { url: String, alt: Option<String> },
}

enum ToolOutput {
    Text(String),
    Error(String),
    Stream(StreamData),
}
```

---

## 5. Error Handling

### 5.1 Error Categories

| Category | HTTP Code | 说明 |
|----------|-----------|------|
| UserInput | 400 | 无效的用户输入 |
| Permission | 403 | 权限不足 |
| NotFound | 404 | 资源不存在 |
| Conflict | 409 | 操作冲突 |
| RateLimit | 429 | 请求过于频繁 |
| Internal | 500 | 服务器内部错误 |

### 5.2 Error Response Format

```json
{
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "User denied permission for Bash tool execution",
    "details": {
      "tool": "Bash",
      "command": "rm -rf /"
    },
    "request_id": "req-123"
  }
}
```

---

## 6. Dependencies

### 6.1 External Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| tokio | 1.x | 异步运行时 |
| serde | 1.x | 序列化 |
| anyhow | 1.x | 错误处理 |
| reqwest | 0.11 | HTTP 客户端 |

### 6.2 Internal Dependencies

| Module | Type | Purpose |
|--------|------|---------|
| Spec-002 | Strong | CLI 是唯一入口 |
| Spec-003 | Strong | 工具是执行能力 |
| Spec-004 | Strong | 权限是安全保障 |
| Spec-006 | Strong | 上下文是理解基础 |
| Spec-007 | Weak | 记忆提供个性化 |

---

## 7. Success Criteria

### 7.1 Functional Criteria

| ID | Criteria | Test Method |
|----|----------|-------------|
| SC-001 | 用户可以启动会话并完成简单任务 | E2E 测试 |
| SC-002 | 会话状态正确管理 | 单元测试 |
| SC-003 | 错误信息清晰可操作 | 人工评审 |
| SC-004 | 所有核心路径有日志 | 代码审查 |

### 7.2 Performance Criteria

| ID | Criteria | Target |
|----|----------|--------|
| PC-001 | 启动时间 | < 2s |
| PC-002 | 首次响应时间 | < 5s |
| PC-003 | 内存使用 | < 500MB |

### 7.3 Security Criteria

| ID | Criteria |
|----|----------|
| SEC-001 | 所有破坏性操作需要确认 |
| SEC-002 | 敏感路径受保护 |
| SEC-003 | 不记录敏感信息到日志 |

---

## 8. Edge Cases

### 8.1 Handled Edge Cases

| Case | Handling |
|------|----------|
| 空代码库 | 显示欢迎信息，引导用户 |
| 超大文件 (>10MB) | 分块读取，显示警告 |
| 网络中断 | 自动重试，缓存结果 |
| 并发修改 | 乐观锁检测 |
| 无效 UTF-8 | 尝试修复或拒绝 |

### 8.2 Known Limitations

| Limitation | Workaround |
|------------|------------|
| 二进制文件 | 显示提示，不尝试读取 |
| 网络超时 | 可配置超时时间 |
| 内存限制 | Token 预算限制 |

---

## 9. Implementation Notes

### 9.1 File Organization

```
devil-agent/
├── src/
│   ├── main.rs              # 入口点
│   ├── cli/                 # CLI 命令处理
│   ├── agent/                # Agent 核心
│   │   ├── mod.rs
│   │   ├── session.rs       # 会话管理
│   │   ├── state.rs         # 状态机
│   │   └── loop.rs          # 主循环
│   ├── tools/               # 工具系统
│   ├── context/             # 上下文管理
│   └── providers/            # LLM 提供者
└── tests/
    └── e2e/
```

### 9.2 Configuration

```toml
# ~/.devil/config.toml
[agent]
name = "devil"
model = "claude-sonnet-4-20250514"
max_turns = 100

[tools]
timeout_seconds = 300
allow_destructive = false

[logging]
level = "info"
format = "json"
```
