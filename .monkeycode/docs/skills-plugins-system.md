# 技能系统与插件架构设计文档

## 概述

### 技能系统（Skills）

Skill 是将**Prompt + 权限配置**封装为可复用的 Markdown 文件。一个代码审查 Skill 不需要审查引擎，只需告诉 AI "审查什么、按什么顺序、输出什么格式"——Skill 把这种"经验"封装为可复用的 Markdown。

### 插件系统（Plugins）

插件系统允许扩展 Claude Code 的功能，支持 Skills、Tools、MCP 服务器、主题等插件类型。

## Skill vs Tool

| 特性 | Tool | Skill |
|------|------|-------|
| 粒度 | 单个原子操作（读文件、执行命令） | 一套完整的工作流（代码审查、创建 PR） |
| 触发方式 | AI 自主选择 | 用户 `/skill-name` 或 AI 通过 `SkillTool` 自动匹配 |
| 本质 | TypeScript 执行逻辑 | **Prompt + 权限配置**的声明式封装 |
| 注册位置 | `src/tools.ts` → `getTools()` | `src/commands.ts` → `getCommands()` |
| 执行器 | 各 Tool 的 `call()` 方法 | `SkillExecutor.call()` → 两条分支（inline / fork） |

## 技能系统

### 五个来源与加载链路

#### 1. 内置命令（Built-in Commands）

硬编码，包含 70+ 条命令（`/commit`、`/review`、`/compact` 等）。

#### 2. Bundled Skills（编译时打包）

关键特性：
- **延迟文件提取**：如果 Skill 声明了 `files`，首次调用时才解压到临时目录
- **闭包级 memoize**：并发调用共享同一个 extraction promise
- 来源标记为 `source: 'bundled'`，在 Prompt 预算中享有**不可截断**的特权

#### 3. 磁盘 Skills（`.claude/skills/`）

加载路径：
```
管理策略：$MANAGED_DIR/.claude/skills/     (Managed)
用户全局：~/.claude/skills/                (UserSettings)
项目级：.claude/skills/                    (ProjectSettings, 向上遍历至 home)
附加目录：--add-dir 指定的路径下 .claude/skills/
```

**加载协议**：只识别 `skill-name/SKILL.md` 目录格式。

#### 4. MCP Skills（动态发现）

通过 `skill://` URI 方案发现并转换为 Command 对象。

**安全边界**：MCP Skills 的 Prompt 内容**禁止执行内联 shell 命令**。

#### 5. Legacy Commands（`/commands/` 目录）

向后兼容的旧格式。

### Frontmatter 字段

```yaml
---
name: code-review                    # 显示名称
description: 系统性代码审查           # 描述
when_to_use: "用户说审查代码、找 bug"  # AI 自动匹配依据
allowed-tools:                       # 工具白名单
  - Read
  - Grep
  - Glob
argument-hint: "<file-or-directory>" # 参数提示
arguments: [path]                    # 声明式参数名
model: opus                          # 模型覆盖
effort: high                         # 努力级别
context: fork                        # 执行模式：inline|fork
agent: code-reviewer                 # 指定 Agent
user-invocable: true                 # 用户是否可 / 调用
disable-model-invocation: false      # 禁止 AI 自主调用
version: "1.0"                       # 版本号
paths:                               # 条件激活的文件路径模式
  - "src/**/*.ts"
hooks:                               # Hook 配置
  PreToolUse:
    - command: ["echo", "checking"]
shell: ["bash"]                      # Shell 执行环境
---
```

### 两条执行路径

#### Inline 模式（默认）

Skill 的 Prompt 内容被注入为 **UserMessage**，在主对话流中继续执行：

1. 处理参数替换（`$ARGUMENTS`）和 shell 命令展开
2. `${CLAUDE_SKILL_DIR}` 被替换为 Skill 所在目录
3. `${CLAUDE_SESSION_ID}` 被替换为当前会话 ID
4. 返回 `new_messages` + `context_modifier`

`context_modifier` 作用：
- **工具白名单注入**：将 `allowed_tools` 合并到 `always_allow_rules`
- **模型切换**：处理模型覆盖
- **努力级别覆盖**：修改 `effort_value`

#### Fork 模式（`context: fork`）

Skill 在**独立子 Agent** 中执行：
1. 构建隔离的 Agent 定义和 Prompt
2. 启动子 Agent 循环，拥有独立的 token 预算
3. 报告工具使用进度
4. 提取结果文本，释放子 Agent 消息

### 权限模型：五层检查

```
1. Deny 规则匹配（支持精确匹配和 prefix:* 通配符）
   ↓ 未命中
2. 远程 canonical Skill 自动放行
   ↓ 未命中
3. Allow 规则匹配
   ↓ 未命中
4. Safe Properties 白名单检查
   ↓ 有非安全属性
5. Ask 用户确认
```

**Safe Properties** 白名单包含 30 个属性名。任何不在白名单中的有意义的属性值都会触发权限请求。这是**正向安全**设计。

### Prompt 预算截断

| 参数 | 说明 |
|------|------|
| **预算计算** | `context_window_tokens × 4 chars/token × 1%` |
| **单条上限** | `MAX_LISTING_DESC_CHARS = 250` 字符 |
| **Bundled** | 不可截断，始终保留完整描述 |
| **降级策略** | 完整描述 → 均分预算 → 仅保留名称 |

### 动态发现与条件激活

#### 基于文件路径的动态发现

从被操作的文件路径开始，**向上遍历**至 CWD：
1. 在每层查找 `.claude/skills/` 目录
2. 使用 `realpath` 去重
3. 按路径深度排序（**深层优先**）

#### 条件激活（paths frontmatter）

带有 `paths` 模式的 Skill 在加载时存入 `conditional_skills` Map。当被操作的文件路径匹配某个 Skill 的 paths 模式时，该 Skill 被**激活**。

### 使用频率排名

```
score = usage_count × max(0.5^(days_since_use / 7), 0.1)
```

- **7 天半衰期**
- **最低 0.1 保底**
- **60 秒去抖**

## 插件系统

### 插件来源

| 来源 | 路径 | 说明 |
|------|------|------|
| **管理策略** | `$MANAGED_DIR/.claude/plugins/` | 企业 managed 插件 |
| **全局目录** | `~/.claude/plugins/` | 用户安装的插件 |
| **项目级** | `.claude/plugins/` | 项目特定插件 |
| **开发模式** | 符号链接 | 开发中插件 |

### 插件类型

```rust
enum PluginType {
    Skills,      // 贡献 Skills
    Tools,       // 贡献 Tools
    MCP,         // MCP 服务器包装
    Theme,       // 主题插件
    Extension,   // 通用扩展
}
```

### 安全策略

三层防护：

| 层级 | 机制 | 说明 |
|------|------|------|
| **黑名单** | `PluginBlocklist` | 阻止已知恶意插件 |
| **白名单模式** | `allow_managed_only` | 仅允许托管插件 |
| **权限级别** | `PermissionLevel` | 限制插件能力 |

### 权限级别

```rust
enum PermissionLevel {
    None,   // 仅读操作
    Low,    // 读 + 网络
    Medium, // 读 + 写工作目录
    High,   // 完全文件系统访问
    Full,   // 包括执行任意代码
}
```

### 自动更新

更新策略：
- 自动检查更新（可配置间隔）
- 忽略重大版本更新（可选）
- 回滚机制（更新失败时）

### 版本控制

SemVer 2.0.0 格式：
- **主版本号**：不兼容的 API 变更
- **次版本号**：向后兼容的功能新增
- **修订号**：向后兼容的问题修正

## 完整生命周期

### Skill 生命周期

```
磁盘 SKILL.md
  ↓ parse_frontmatter()
  ↓ parse_skill_frontmatter_fields() → 16 个字段
  ↓ create_skill_command() → Command 对象
  ↓ 去重（realpath + seen_file_ids）
  ↓ 条件 Skill → conditional_skills Map
  ↓ get_all_commands() 合并 local + MCP
  ↓ format_commands_within_budget() → 截断注入 System Prompt
  ↓ AI 选择匹配的 Skill
  ↓ validate_input() → 名称校验 + 存在性检查
  ↓ check_permissions() → 五层权限检查
  ↓ execute() → inline 或 fork 执行
  ↓ context_modifier() → 注入 allowed_tools + model + effort
  ↓ record_skill_usage() → 更新使用频率排名
```

### 插件生命周期

```
插件目录扫描
  ↓ 读取 package.json/plugin.json
  ↓ 解析元数据
  ↓ 确定位置（Global/Project/Managed/Development）
  ↓ 黑名单检查
  ↓ 安全策略检查
  ↓ 加载到注册表
  ↓ 定期检查更新
  ↓ 自动/手动更新
```

## 使用示例

### Skill 使用

```rust
use agent::skills::{SkillLoader, SkillExecutor, SkillPermissionChecker};

// 加载 Skills
let mut loader = SkillLoader::new();
loader.load_all_from_disk().unwrap();

// 激活匹配路径的 Skills
let activated = loader.activate_skills_for_path("src/auth/validate.ts");

// 执行 Skill
let executor = SkillExecutor::new("session-123");
let result = executor.execute(&skill, Some("arg1 arg2")).await.unwrap();

// 权限检查
let checker = SkillPermissionChecker::new()
    .with_allow_rules(vec![/* ... */])
    .with_deny_rules(vec![/* ... */]);
let permission = checker.check(&skill);
```

### 插件使用

```rust
use agent::plugins::{PluginLoader, PluginBlocklist, PluginSecurityPolicy};

// 创建插件加载器
let mut loader = PluginLoader::new()
    .with_plugin_dirs(vec![
        PathBuf::from("~/.claude/plugins"),
        PathBuf::from(".claude/plugins"),
    ])
    .with_blocklist(PluginBlocklist::new())
    .with_security_policy(PluginSecurityPolicy {
        allow_managed_only: false,
        ..Default::default()
    });

// 加载所有插件
let count = loader.load_all().unwrap();

// 获取已加载的插件
for plugin in loader.get_plugins() {
    println!("Plugin: {} v{}", plugin.metadata.name, plugin.metadata.version);
}
```

## 文件索引

| 文件 | 职责 |
|------|------|
| `crates/agent/src/skills/mod.rs` | 技能系统模块入口 |
| `crates/agent/src/skills/types.rs` | Skill 类型定义 |
| `crates/agent/src/skills/loader.rs` | Skill 加载器 |
| `crates/agent/src/skills/executor.rs` | Skill 执行引擎 |
| `crates/agent/src/skills/permissions.rs` | Skill 权限模型 |
| `crates/agent/src/plugins/mod.rs` | 插件系统模块入口 |
| `crates/agent/src/plugins/types.rs` | 插件类型定义 |
| `crates/agent/src/plugins/loader.rs` | 插件加载器 |

## 与 Claude Code 对齐

| Claude Code 文件 | 本实现 |
|-----------------|--------|
| `src/skills/loadSkillsDir.ts` | `skills/loader.rs` |
| `src/tools/SkillTool/SkillTool.ts` | `skills/executor.rs` |
| `docs/extensibility/skills.mdx` | 本文档 |
| `src/utils/plugins/pluginLoader.ts` | `plugins/loader.rs` |
| `src/utils/plugins/pluginIdentifier.ts` | `plugins/types::plugin_identifier` |
| `src/utils/plugins/pluginBlocklist.ts` | `plugins/types::PluginBlocklist` |

## 待办事项

- [ ] 实现 Bundled Skills 解压
- [ ] 实现 MCP Skills 获取
- [ ] 实现完整的 Shell 命令展开
- [ ] 实现技能预算管理和截断逻辑
- [ ] 实现插件自动更新
- [ ] 实现插件签名验证
- [ ] 集成到 Agent 主循环
- [ ] 编写单元测试和集成测试
