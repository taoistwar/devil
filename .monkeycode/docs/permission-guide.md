# 权限系统使用指南

## 快速开始

### 1. 配置项目级权限

在项目根目录创建 `.claude/settings.json`：

```json
{
  "permissions": {
    "mode": "default",
    "allow": [
      "Bash(npm test)",
      "Bash(npm run lint)",
      "Bash(git:*)",
      "Read",
      "Glob",
      "Grep"
    ],
    "deny": [
      "Bash(npm publish)",
      "Bash(rm -rf *)",
      "Bash(* > /etc/*)"
    ]
  }
}
```

### 2. 权限规则语法

#### 精确匹配

```json
"Bash(npm test)"  // 仅允许 npm test 这一条命令
```

#### 前缀匹配（推荐）

```json
"Bash(npm:*)"     // 允许所有以 npm 开头的命令
                   // 如：npm test, npm run build, npm install
```

#### 通配符匹配

```json
"Bash(git *)"     // 匹配 git 后跟任意参数
                   // 如：git status, git commit -m "fix"
```

**注意**：`Bash(git:*)` 和 `Bash(git *)` 是等价的，都匹配所有 git 命令。

#### 工具级规则

```json
"Read"            // 允许所有 Read 工具调用
"Glob"            // 允许所有 Glob 工具调用
```

#### MCP 服务器级通配符

```json
"mcp__server1__*" // 匹配 server1 服务器下的所有工具
```

### 3. 规则来源优先级

优先级从高到低：

1. **Session**（会话级） - 当前会话临时设置，最高优先级
2. **Command**（命令级） - 启动命令附带的规则
3. **CliArg**（命令行参数） - CLI 参数指定的规则
4. **PolicySettings**（策略设置） - 企业策略强制规则
5. **FlagSettings**（功能标志） - 功能开关相关规则
6. **LocalSettings**（本地设置） - `.claude/settings.local.json`，不提交到 Git
7. **ProjectSettings**（项目设置） - `.claude/settings.json`，团队共享
8. **UserSettings**（全局用户设置） - 用户家目录的全局配置，最低优先级

**优先级铁律**：deny 规则的优先级最高，无论来源如何。

### 4. 权限模式

#### Default 模式（默认）

```json
{ "permissions": { "mode": "default" } }
```

- 每次工具调用都需要用户确认（除非被 allow 规则放行）
- 适用场景：日常交互式开发

#### Plan 模式

```json
{ "permissions": { "mode": "plan" } }
```

- 只允许只读操作（Read、Grep、Glob）
- 写入操作（Edit、Write、Bash）被拒绝
- 适用场景：代码审查、架构分析

#### Auto 模式

```json
{ "permissions": { "mode": "auto" } }
```

- AI 分类器自动审批常见操作
- 安全工具（Read、Grep、Glob、TodoWrite）跳过检查
- 适用场景：熟悉的工作流，信任 AI 判断

**注意**：auto 模式不适合以下场景：
- 生产环境部署
- 敏感数据操作（密钥、凭证）
- 不可逆操作（删除数据库）

#### Bypass 模式

```json
{ "permissions": { "mode": "bypassPermissions" } }
```

- 除 deny 规则外全部自动放行
- 适用场景：CI/CD 管道、自动化测试

**企业级安全建议**：
1. 在容器或虚拟机中运行
2. 配置显式的 deny 规则
3. 使用 `--allowedTools` 限制工具范围
4. 启用审计日志

### 5. 本地个人配置

创建 `.claude/settings.local.json`（不提交到 Git）：

```json
{
  "permissions": {
    "allow": [
      "Bash(npx eslint *)",
      "Bash(cargo check)",
      "Bash(cargo clippy)"
    ]
  }
}
```

使用 `.gitignore` 排除：

```gitignore
# 在 .gitignore 中添加
.claude/settings.local.json
```

## 编程 API

### 创建权限上下文

```rust
use agent::permissions::*;

// 创建默认上下文
let mut ctx = ToolPermissionContext::with_defaults();

// 或者自定义模式
let mut ctx = ToolPermissionContext::new(PermissionMode::Auto);
```

### 添加规则

```rust
// 添加 allow 规则
ctx.add_allow_rule(
    RuleSource::UserSettings,
    "Bash(npm:*)"
);

// 添加 deny 规则
ctx.add_deny_rule(
    RuleSource::ProjectSettings,
    "Bash(rm -rf *)"
);

// 添加 ask 规则
ctx.add_ask_rule(
    RuleSource::Session,
    "Bash(npm publish)"
);
```

### 执行权限检查

```rust
use agent::permissions::PermissionPipeline;

let pipeline = PermissionPipeline;

let result = pipeline.check_permissions(
    &tool,           // 工具实例
    &input,          // 工具输入（JSON）
    &tool_context,   // 工具执行上下文
    &perm_context,   // 权限上下文
).await?;

match result.behavior {
    PermissionBehavior::Allow => {
        // 执行工具
        println!("Operation allowed");
    }
    PermissionBehavior::Deny { reason } => {
        // 拒绝并提示原因
        eprintln!("Denied: {}", reason);
    }
    PermissionBehavior::Ask { prompt } => {
        // 请求用户确认
        println!("Confirm: {}", prompt);
        // 等待用户响应...
    }
    PermissionBehavior::Passthrough => {
        // 交由后续阶段决定（通常会变为 ask）
        println!("Requires confirmation");
    }
}
```

### 权限更新

```rust
// 创建更新操作
let updates = vec![
    PermissionUpdate::add_rules(
        RuleSource::UserSettings,
        vec![PermissionRule::allow(
            RuleSource::UserSettings,
            "Glob"
        )]
    ),
    PermissionUpdate::set_mode(PermissionMode::Auto),
];

// 应用更新（内存中）
let result = apply_permission_updates(&ctx, &updates);

// 检查是否有可持久化的更新
if result.has_persistable_update {
    // 持久化到文件系统
    // persist_permission_updates(&result.new_context)?;
}

// 使用更新后的上下文
let new_ctx = result.new_context;
```

### Bash 命令分析

```rust
use agent::permissions::bash_analyzer::BashSemanticAnalyzer;

let analysis = BashSemanticAnalyzer::analyze_command("git status");

println!("Is search: {}", analysis.is_search);
println!("Is read: {}", analysis.is_read);
println!("Is list: {}", analysis.is_list);
println!("Is destructive: {}", analysis.is_destructive);
println!("Is dangerous: {}", analysis.is_dangerous);
println!("Can sandbox: {}", analysis.can_sandbox);

if analysis.is_dangerous {
    println!("Danger reason: {:?}", analysis.danger_reason);
}
```

## 实战场景

### 场景 1：Node.js 项目开发

**需求**：
- 自动运行 `npm test` 和 `npm run lint`
- 允许所有 `git` 命令
- 禁止 `npm publish`
- 禁止删除 `node_modules` 之外的目录

**配置**：

```json
{
  "permissions": {
    "mode": "auto",
    "allow": [
      "Bash(npm test)",
      "Bash(npm run lint)",
      "Bash(git:*)",
      "Read",
      "Glob",
      "Grep"
    ],
    "deny": [
      "Bash(npm publish)",
      "Bash(rm -rf !node_modules/)"
    ]
  }
}
```

### 场景 2：代码审查

**需求**：
- 只允许读取操作
- 不允许任何写入或执行

**配置**：

```json
{
  "permissions": {
    "mode": "plan"
  }
}
```

或者显式配置：

```json
{
  "permissions": {
    "allow": [
      "Read",
      "Glob",
      "Grep"
    ],
    "deny": [
      "Edit",
      "Write",
      "Bash"
    ]
  }
}
```

### 场景 3：CI/CD 自动化

**需求**：
- 完全自动化，无需人工确认
- 但仍需阻止危险操作

**配置**：

```json
{
  "permissions": {
    "mode": "bypassPermissions",
    "deny": [
      "Bash(rm -rf /)",
      "Bash(* > /etc/*)",
      "Bash(sudo *)",
      "Bash(npm publish)"
    ]
  }
}
```

### 场景 4：团队协作配置

**团队共享配置**（`.claude/settings.json`）：

```json
{
  "permissions": {
    "allow": [
      "Bash(npm:*)",
      "Bash(yarn:*)",
      "Bash(pnpm:*)",
      "Bash(cargo check)",
      "Bash(cargo clippy)",
      "Bash(go test ./...)",
      "Read",
      "Glob",
      "Grep"
    ],
    "deny": [
      "Bash(* publish)",
      "Bash(* deploy *)",
      "Bash(git push --force)"
    ]
  }
}
```

**个人偏好配置**（`.claude/settings.local.json`）：

```json
{
  "permissions": {
    "allow": [
      "Bash(docker compose *)",
      "Bash(kubectl get *)",
      "Bash(kubectl describe *)"
    ]
  }
}
```

## 常见问题

### Q1: `Bash(npm run *)` 是否匹配 `npm run`（不带子命令）？

**答**：是的。当模式以 ` *`（空格加通配符）结尾且这是唯一的通配符时，尾部空格和参数变为可选的。

### Q2: 如何禁止访问特定目录？

**答**：使用通配符规则：

```json
{
  "deny": [
    "Bash(* /etc/*)",
    "Bash(* /var/*)",
    "Bash(* /usr/*)"
  ]
}
```

### Q3: 如何允许子目录的删除但禁止根目录删除？

**答**：使用精确的 deny 规则：

```json
{
  "allow": [
    "Bash(rm -rf ./tmp/*)",
    "Bash(rm -rf ./build/*)"
  ],
  "deny": [
    "Bash(rm -rf /)",
    "Bash(rm -rf /*)",
    "Bash(rm -rf ~)"
  ]
}
```

### Q4: 规则冲突时如何处理？

**答**：遵循优先级铁律：
1. deny 永远优先于 allow
2. 高优先级来源覆盖低优先级来源
3. 同一来源内，后设置的规则覆盖 prior 规则

### Q5: 如何临时允许一次操作？

**答**：在交互式提示中选择 "Allow once"（本次允许），不会持久化到配置文件。

## 最佳实践

### 1. 分层配置

- **团队共享规则** → `.claude/settings.json`（提交到 Git）
- **个人偏好规则** → `.claude/settings.local.json`（不提交到 Git）
- **临时规则** → 会话级更新（重启后失效）

### 2. 使用前缀匹配

优先使用前缀匹配（`:*` 语法），语义更清晰：

```json
// 推荐
"Bash(npm:*)"

// 不推荐（容易混淆）
"Bash(npm *)"
```

### 3. 宽泛 allow + 精确 deny

先用宽泛的 allow 规则授权基本操作，再用精确的 deny 规则排除危险操作：

```json
{
  "allow": [
    "Bash(git:*)"
  ],
  "deny": [
    "Bash(git push --force)"
  ]
}
```

### 4. 定期审查规则

定期检查 `.claude/settings.json` 中的规则，移除不再需要的允许规则。

### 5. 使用 auto 模式要谨慎

auto 模式适合熟悉的工作流，但不适用于：
- 新项目的初始开发
- 涉及敏感数据的操作
- 生产环境部署

## 参考资料

- 权限系统设计文档：`.monkeycode/docs/permission-system.md`
- 实现总结：`.monkeycode/docs/permission-implementation.md`
- 《御舆：解码 Agent Harness》第四章
- https://lintsinghua.github.io/
