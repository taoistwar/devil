# 快速开始指南

本指南帮助您快速搭建开发环境并开始使用 MonkeyCode AI Agent 框架。

## 环境要求

### 必需软件

| 软件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Rust | 1.70.0 | 1.75.0+ |
| Cargo | 1.70.0 | 1.75.0+ |
| Git | 2.0 | 2.40+ |

### 可选软件

| 软件 | 用途 |
|------|------|
| Anthropic API Key | 用于真实模型调用 |
| SQLite | 用于持久化存储 |
| Docker | 用于容器化部署 |

## 安装 Rust

### Linux/macOS

```bash
# 使用 rustup 安装
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 加载环境变量
source $HOME/.cargo/env

# 验证安装
rustc --version
cargo --version
```

### Windows

1. 下载并运行 [rustup-init.exe](https://win.rustup.rs/x86_64)
2. 按照安装向导完成安装
3. 打开新的 PowerShell 窗口验证：

```powershell
rustc --version
cargo --version
```

## 获取项目

```bash
# 克隆仓库
git clone <repository-url>
cd <project-directory>

# 或初始化为新项目
cargo new my-agent --lib
cd my-agent

# 添加依赖
cargo add agent channels memory plugins providers gateway
```

## 构建项目

```bash
# 构建所有 crate（debug 模式）
cargo build

# 构建所有 crate（release 模式，优化）
cargo build --release

# 构建特定 crate
cargo build -p agent

# 构建并生成文档
cargo doc --open
```

## 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定 crate 的测试
cargo test -p agent

# 运行测试并显示输出
cargo test -- --nocapture

# 运行特定测试
cargo test test_agent_creation
```

## 配置项目

### Workspace 配置

根目录的 `Cargo.toml` 定义了 workspace：

```toml
[workspace]
resolver = "2"
members = [
    "crates/channels",
    "crates/agent",
    "crates/memory",
    "crates/plugins",
    "crates/gateway",
    "crates/providers",
]

[workspace.package]
version = "0.1.0"
edition = "2021"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1"
thiserror = "1"
chrono = "0.4"
reqwest = "0.11.14"
uuid = { version = "1", features = ["v4"] }
```

### Crate 配置示例

```toml
[package]
name = "my-agent"
version.workspace = true
edition.workspace = true

[dependencies]
agent = { path = "../crates/agent" }
channels = { path = "../crates/channels" }
tokio.workspace = true
serde.workspace = true
anyhow.workspace = true
```

## 快速上手

### 示例 1: 基本 Agent 使用

```rust
use agent::{Agent, AgentConfigBuilder, Message};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    // 创建 Agent 配置
    let config = AgentConfigBuilder::new()
        .name("my-assistant")
        .model("claude-sonnet-4-20250514")
        .system_prompt("你是一个有帮助的 AI 助手，使用中文回复。")
        .max_turns(50)
        .max_context_tokens(200000)
        .enable_streaming_tools(true)
        .build();
    
    // 创建 Agent 实例
    let agent = Agent::new(config.clone())?;
    
    // 初始化
    agent.initialize().await?;
    println!("Agent {} 已初始化", config.name);
    
    // 执行单次对话
    let user_message = Message::user_text("你好，请介绍一下你自己");
    let result = agent.run_once(user_message).await?;
    
    println!("对话完成，共 {} 轮", result.turn_count);
    println!("终止原因：{:?}", result.terminal.reason);
    
    // 关闭 Agent
    agent.shutdown().await?;
    Ok(())
}
```

### 示例 2: 使用自定义工具

```rust
use agent::{Agent, AgentConfigBuilder, Message, Tool, ToolContext, ToolResult, ToolPermissionLevel};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;

// 定义自定义工具
struct FileReadTool;

#[async_trait]
impl Tool for FileReadTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "读取文件内容"
    }

    fn input_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "文件路径"
                }
            },
            "required": ["path"]
        })
    }

    fn permission_level(&self) -> ToolPermissionLevel {
        ToolPermissionLevel::ReadOnly
    }

    async fn execute(&self, input: serde_json::Value, _ctx: &ToolContext) -> Result<ToolResult> {
        let path = input["path"].as_str().unwrap_or("");
        
        // 这里简化处理，实际应该读取文件
        let content = format!("文件 {} 的内容", path);
        
        Ok(ToolResult::success("read-1", content))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let config = AgentConfigBuilder::new()
        .name("file-agent")
        .model("claude-sonnet-4-20250514")
        .system_prompt("你是一个文件分析助手。")
        .max_turns(10)
        .build();
    
    let agent = Agent::new(config)?;
    agent.initialize().await?;
    
    // 注册工具
    agent.register_tool(FileReadTool).await?;
    
    // 执行对话
    let message = Message::user_text("请读取 src/main.rs 文件并分析内容");
    let result = agent.run_once(message).await?;
    
    println!("分析完成");
    agent.shutdown().await?;
    Ok(())
}
```

### 示例 3: 使用测试依赖

```rust
use agent::{Agent, AgentConfig, TestDeps, Message, ModelCallParams, ModelCallResult, AssistantMessage};
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = AgentConfig::default();
    
    // 创建测试依赖，注入自定义行为
    let test_deps = Arc::new(TestDeps::new(
        // 自定义模型调用
        |params| {
            Ok(ModelCallResult {
                assistant_message: AssistantMessage::text(format!(
                    "收到 {} 条消息",
                    params.messages.len()
                )),
                input_tokens: 100,
                output_tokens: 50,
                stop_reason: Some("stop_sequence".to_string()),
            })
        },
        // 自定义 micro_compact
        |msgs| Ok(agent::deps::CompactResult {
            messages: msgs,
            success: true,
            token_reduction: Some(1000),
        }),
        // 自定义 auto_compact
        |msgs| Ok(agent::deps::CompactResult {
            messages: msgs,
            success: false,
            token_reduction: None,
        }),
        // 自定义 UUID 生成
        || "test-uuid-123".to_string(),
    ));
    
    // 使用测试依赖创建 Agent
    let agent = Agent::with_deps(config, test_deps);
    agent.initialize().await?;
    
    let result = agent.run_once(Message::user_text("Hello")).await?;
    println!("测试完成：{} 轮", result.turn_count);
    
    Ok(())
}
```

### 示例 4: 使用记忆系统

```rust
use memory::{MemoryManager, MemoryEntry};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // 创建记忆管理器
    let manager = MemoryManager::new(100);
    
    // 添加短期记忆
    let entry = MemoryEntry {
        id: "user-pref-1".to_string(),
        content: "用户偏好中文回复".to_string(),
        timestamp: 1234567890,
        tags: vec!["preference".to_string(), "language".to_string()],
    };
    
    manager.short_term().add(entry.clone()).await?;
    
    // 检索记忆
    if let Some(memory) = manager.short_term().get("user-pref-1").await {
        println!("检索到记忆：{}", memory.content);
    }
    
    // 长期存储
    manager.long_term().store(entry).await?;
    
    // 按标签搜索
    let results = manager.long_term()
        .search_by_tags(&vec!["preference".to_string()])
        .await;
    
    println!("找到 {} 条相关记忆", results.len());
    
    Ok(())
}
```

### 示例 5: 使用插件系统

```rust
use plugins::{PluginManager, Plugin, PluginContext, PluginMetadata, PluginResult};
use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

// 定义自定义插件
struct DataProcessorPlugin;

#[async_trait]
impl Plugin for DataProcessorPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "data-processor".to_string(),
            version: "1.0.0".to_string(),
            description: "数据处理插件".to_string(),
            author: Some("Developer".to_string()),
        }
    }

    async fn initialize(&mut self) -> Result<()> {
        println!("插件初始化");
        Ok(())
    }

    async fn execute(&self, ctx: PluginContext) -> Result<PluginResult> {
        // 处理输入数据
        let output = serde_json::json!({
            "processed": true,
            "input": ctx.input
        });
        
        Ok(PluginResult {
            success: true,
            output: Some(output),
            error: None,
        })
    }

    async fn shutdown(&self) -> Result<()> {
        println!("插件关闭");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut manager = PluginManager::new();
    
    // 注册插件
    manager.register(DataProcessorPlugin).await?;
    
    // 初始化所有插件
    manager.initialize_all().await?;
    
    // 执行插件
    let ctx = PluginContext {
        request_id: "req-1".to_string(),
        input: serde_json::json!({"data": [1, 2, 3]}),
        env: HashMap::new(),
    };
    
    let result = manager.execute_plugin("data-processor", ctx).await?;
    println!("插件执行结果：{:?}", result);
    
    // 列出所有插件
    let plugins = manager.list_plugins().await;
    for plugin in plugins {
        println!("插件：{} v{}", plugin.name, plugin.version);
    }
    
    // 关闭所有插件
    manager.shutdown_all().await?;
    
    Ok(())
}
```

## 开发工作流

### 代码格式化

```bash
# 格式化代码
cargo fmt

# 检查格式
cargo fmt -- --check
```

### 代码检查

```bash
# 检查代码（不生成可执行文件）
cargo check

# 使用 clippy 进行 lint 检查
cargo clippy

# 自动修复 clippy 警告
cargo clippy --fix
```

### 依赖管理

```bash
# 更新依赖
cargo update

# 检查过期依赖
cargo outdated  # 需要 cargo install cargo-outdated

# 检查未使用依赖
cargo udeps     # 需要 cargo install cargo-udeps
```

### 性能分析

```bash
# 构建 release 版本进行性能测试
cargo build --release

# 生成性能分析报告
cargo flamegraph --bin my-agent  # 需要 cargo install flamegraph
```

## 常见问题

### Q: 编译速度慢怎么办？

```bash
# 启用并行编译
echo '[build]
jobs = 4' >> ~/.cargo/config.toml

# 使用 mold 链接器（Linux）
cargo install cargo-mold
cargo mold run
```

### Q: 如何调试异步代码？

```rust
// 使用 tokio 控制台
cargo add tokio-console

// 在代码中添加
#[tokio::main]
async fn main() {
    console_subscriber::init();
    // ...
}

// 运行后使用 tokio-console 查看
tokio-console
```

### Q: 如何处理大文件？

```rust
// 使用流式处理
use tokio::io::{AsyncReadExt, BufReader};

let file = tokio::fs::File::open("large_file.txt").await?;
let mut reader = BufReader::with_capacity(8 * 1024, file);

let mut buffer = Vec::new();
reader.read_to_end(&mut buffer).await?;
```

### Q: 如何配置日志级别？

```rust
// 方法 1: 代码配置
use tracing_subscriber::{fmt, EnvFilter};

fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

// 方法 2: 环境变量
RUST_LOG=debug cargo run
RUST_LOG=agent=debug,cargo run
```

## 下一步

完成本指南后，您可以：

1. 阅读 [架构设计文档](architecture.md) 了解详细架构
2. 阅读 [Crate 参考](crates.md) 了解各模块 API
3. 查看示例代码学习最佳实践
4. 开始实现您的第一个 Agent

## 获取帮助

- 查看 [GitHub Issues](https://github.com/your-repo/issues) 报告问题
- 查看 [Discussions](https://github.com/your-repo/discussions) 参与讨论
- 阅读源码中的文档注释
