# Devil MCP

MCP (Model Context Protocol) 集成模块 - AI 世界的"USB-C 接口"

## 架构概览

MCP 定义了 AI 应用与外部数据源和工具之间的统一标准协议。通过 MCP，Devil Agent 可以以统一的方式连接任何支持 MCP 的服务器，包括：

- **filesystem** - 文件系统访问
- **github** - GitHub API
- **gitlab** - GitLab API  
- **slack** - Slack 消息
- **linear** - Linear 任务管理
- **bigquery** - Google BigQuery
- **fetch** - 网页抓取
- **postgres** - PostgreSQL 数据库

## 已实现模块

### 核心组件

| 模块 | 文件 | 状态 | 说明 |
|------|------|------|------|
| 类型定义 | `types.rs` | ✅ | 服务器配置、连接状态、工具映射 |
| 连接管理器 | `connection_manager.rs` | ✅ | 状态机、企业策略过滤器、7 层配置加载 |
| 传输协议 | `transports/*` | ✅ | 8 种传输协议实现 |
| 工具发现 | `tool_discovery.rs` | ✅ | 工具发现、名称映射、Unicode 清理 |
| 权限检查 | `permissions.rs` | ✅ | 四层权限模型、深度防御 |
| Bridge 通信 | `bridge/*` | ✅ | 双向通信、消息路由、去重 |
| 控制协议 | `control_protocol.rs` | ✅ | initialize/set_model/interrupt 等 |

### 测试覆盖

| 测试类型 | 文件 | 状态 | 说明 |
|----------|------|------|------|
| 单元测试 | 各模块内 `#[cfg(test)]` | ✅ | 每个核心函数的独立测试 |
| 集成测试 | `tests/integration_test.rs` | ✅ | 端到端流程测试 |

### 传输协议实现

| 协议 | 文件 | 说明 | 适用场景 |
|------|------|------|----------|
| Stdio | `transports/stdio.rs` | 标准输入输出子进程 | 本地 MCP 服务器 |
| Streamable HTTP | `transports/streamable_http.rs` | 双工 SSE（推荐） | 远程 MCP 服务器 |
| HTTP Polling | `transports/http_polling.rs` | HTTP 轮询 | 兼容旧服务器 |
| WebSocket | `transports/websocket.rs` | WebSocket 全双工 | 实时通信 |
| SDK (Rust/Python/Node/Bun) | `transports/sdk.rs` | 多语言 SDK 同进程 | 高性能场景 |

## 核心功能

### 1. 连接状态机

5 种连接状态：

```
Connected     - 已连接，可正常使用
Failed        - 连接失败（指数退避重试）
NeedsAuth     - 需要用户认证
Pending       - 等待用户授权
Disabled      - 被策略禁用
```

### 2. 四层权限模型

深度防御安全策略：

```
Level 1: Enterprise Policy  - 企业策略（最严格，全局生效）
Level 2: IDE Whitelist      - IDE 管理员白名单
Level 3: User Permissions   - 用户个性化权限
Level 4: Runtime Confirmation - 运行时用户确认
```

### 3. 工具发现与映射

- **自动发现**: `tools/list` 请求获取服务器工具列表
- **名称映射**: `mcp__{server}__{tool}` 确保全局唯一性
- **Unicode 清理**: 移除控制字符和非法字符
- **缓存管理**: 按服务器索引的工具缓存

### 4. Bridge 双向通信

- **请求 - 响应匹配**: 基于 JSON-RPC ID 的异步通信
- **通知广播**: 支持多订阅者
- **去重系统**: BoundedUUIDSet 自动维护 10000 个最近 UUID
- **心跳超时**: 可配置的请求超时机制

## 使用示例

### 创建连接管理器

```rust
use devil_mcp::{McpConnectionManager, McpServerConfig};

let manager = McpConnectionManager::new();
```

### 加载服务器配置

```rust
// 从 7 层配置作用域加载
let result = manager.load_from_config().await?;

println!("Allowed servers: {:?}", result.allowed);
println!("Filtered servers: {:?}", result.filtered_by_policy);
```

### 重新连接服务器

```rust
let result = manager.reconnect("filesystem").await?;

println!("Discovered {} tools", result.tools.len());
println!("Registered {} commands", result.commands.len());
```

### 权限检查

```rust
use devil_mcp::{PermissionChecker, EnterprisePolicy, UserPermissions};

let checker = PermissionChecker::new(
    EnterprisePolicy::default(),
    IdeWhitelist::default(),
    UserPermissions::default(),
);

match checker.check_server("github").await {
    PermissionResult::Allowed => println!("Server is allowed"),
    PermissionResult::Denied(reason) => println!("Denied: {}", reason),
    PermissionResult::NeedsConfirmation => println!("Needs user confirmation"),
}
```

### 工具调用

```rust
use devil_mcp::{McpBridge, BridgeState};

let bridge = McpBridge::new("filesystem", 100, 30000); // 30s 超时

bridge.start().await?;

// 调用工具
let result = bridge.call_tool(
    "read_file",
    serde_json::json!({"path": "/tmp/test.txt"})
).await?;

println!("Tool result: {:?}", result);
```

## 配置示例

### 服务器配置（JSON）

```json
{
  "mcpServers": {
    "filesystem": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/allowed/path"],
      "env": {"DEBUG": "true"},
      "disabled": false,
      "autoApprove": ["read_file", "list_directory"],
      "namePrefixStrategy": "ServerName"
    },
    "github": {
      "url": "https://api.githubcopilot.com/mcp",
      "type": "streamable-http",
      "token": "${GITHUB_TOKEN}",
      "disabled": false
    },
    "local-llama": {
      "sdkLanguage": "python",
      "sdkConfig": {"model": "llama-3.2", "port": 8080},
      "disabled": false
    }
  }
}
```

### 企业策略（YAML）

```yaml
mcp:
  enabled: true
  blockedServers:
    - "untrusted-*"
  allowedTools:
    - "*::read_*"
    - "filesystem::*"
  requireAdminApproval: true
```

## 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     Devil Agent                              │
├─────────────────────────────────────────────────────────────┤
│  MCP 层                                                      │
├──────────────┬──────────────┬──────────────┬────────────────┤
│  Connection  │  Transport   │  Tool        │  Permission    │
│  Manager     │  Protocols   │  Discovery   │  Checker       │
├──────────────┼──────────────┼──────────────┼────────────────┤
│  - 状态机    │  - stdio     │  - 发现      │  - 企业策略    │
│  - 重试      │  - SSE       │  - 映射      │  - IDE 白名单   │
│  - 配置加载  │  - HTTP      │  - 缓存      │  - 用户权限    │
│  - 过滤器    │  - WS        │  - Unicode   │  - 运行时确认  │
│              │  - SDK       │    清理      │                │
├──────────────┴──────────────┴──────────────┴────────────────┤
│                      MCP Bridge                              │
│  ┌──────────────┬──────────────┬──────────────┐             │
│  │ Message      │  Bounded     │  Control     │             │
│  │ Router       │  UUID Set    │  Protocol    │             │
│  └──────────────┴──────────────┴──────────────┘             │
├─────────────────────────────────────────────────────────────┤
│              MCP Servers (外部)                             │
│  filesystem │ github │ gitlab │ slack │ postgres │ ...      │
└─────────────────────────────────────────────────────────────┘
```

## 开发进度

### 核心模块
- ✅ 类型定义 (`types.rs`)
- ✅ 连接管理器 (`connection_manager.rs`)
- ✅ 传输协议层 (`transports/*`)
  - ✅ Stdio
  - ✅ Streamable HTTP
  - ✅ HTTP Polling
  - ✅ WebSocket
  - ✅ SDK (Rust/Python/Node.js/Bun)
- ✅ 工具发现 (`tool_discovery.rs`)
- ✅ 权限检查 (`permissions.rs`)
- ✅ Bridge 通信 (`bridge/*`)
  - ✅ Message Router
  - ✅ BoundedUUIDSet
- ✅ 控制协议 (`control_protocol.rs`)
  - ✅ initialize
  - ✅ set_model
  - ✅ interrupt
  - ✅ ping/pong
  - ✅ cancel

### 测试覆盖
- ✅ 控制协议单元测试
- ✅ 权限检查单元测试（四层模型测试）
- ✅ 工具发现单元测试（名称映射、缓存管理）
- ✅ Message Router 单元测试（请求路由、通知广播）
- ✅ BoundedUUIDSet 单元测试（去重、边界测试）
- ✅ 集成测试 (`tests/integration_test.rs`)
- 🔄 传输协议集成测试（需要实际 MCP 服务器）
- 🔄 E2E 测试（完整连接流程）

### 文档
- ✅ README.md - 模块概述和使用示例
- ✅ Rustdoc - 内联文档注释

## 下一步计划

1. **传输协议集成测试** - 使用 Mock MCP 服务器测试各传输协议
2. **E2E 测试** - 完整连接流程测试（连接→发现→调用→断开）
3. **性能基准** - 连接池复用、缓存性能测试
4. **错误处理增强** - 完善错误类型和恢复策略
5. **与 Agent 集成** - 将 MCP 模块集成到技能执行引擎

## 参考链接

- [MCP 官方文档](https://modelcontextprotocol.io/)
- [第 12 章设计文档](../../../.monkeycode/specs/2026-04-17-chapter12-mcp-integration/design.md)
- [Claude Code MCP 架构](https://github.com/anthropics/claude-code)
