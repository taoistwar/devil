//! 技能系统模块
//! 
//! 实现 Claude Code Skills 系统：从磁盘加载、Frontmatter 解析、预算感知描述截断、
//! 双模式执行（inline/fork）、权限白名单、条件激活、动态发现到远程技能加载。
//! 
//! ## 概述
//! 
//! Skill 是将**Prompt + 权限配置**封装为可复用的 Markdown 文件。一个代码审查 Skill 不需要审查引擎，
//! 只需告诉 AI "审查什么、按什么顺序、输出什么格式"——Skill 把这种"经验"封装为可复用的 Markdown。
//! 
//! ## Skill vs Tool
//! 
//! | 特性 | Tool | Skill |
//! |------|------|-------|
//! | 粒度 | 单个原子操作（读文件、执行命令） | 一套完整的工作流（代码审查、创建 PR） |
//! | 触发方式 | AI 自主选择 | 用户 `/skill-name` 或 AI 通过 `SkillTool` 自动匹配 |
//! | 本质 | TypeScript 执行逻辑 | **Prompt + 权限配置**的声明式封装 |
//! | 注册位置 | `src/tools.ts` → `getTools()` | `src/commands.ts` → `getCommands()` |
//! | 执行器 | 各 Tool 的 `call()` 方法 | `SkillExecutor.call()` → 两条分支（inline / fork） |
//! 
//! ## 五个来源与加载链路
//! 
//! ### 1. 内置命令（Built-in Commands）
//! 
//! 硬编码，包含 70+ 条命令（`/commit`、`/review`、`/compact` 等）。这些是 TypeScript 模块而非 Markdown，
//! 但实现了相同的 `Command` 接口。
//! 
//! ### 2. Bundled Skills（编译时打包）
//! 
//! 通过 `register_bundled_skill()` 在模块初始化时注册。关键特性：
//! - **延迟文件提取**：如果 Skill 声明了 `files`（参考文件），首次调用时才解压到临时目录
//! - **闭包级 memoize**：并发调用共享同一个 extraction promise，避免竞态写入
//! - 来源标记为 `source: 'bundled'`，在 Prompt 预算中享有**不可截断**的特权
//! 
//! ### 3. 磁盘 Skills（`.claude/skills/`）
//! 
//! 由 `load_skills_from_skills_dir()` 加载，这是最重要的加载路径：
//! 
//! ```
//! 管理策略：$MANAGED_DIR/.claude/skills/     (Managed)
//! 用户全局：~/.claude/skills/                (UserSettings)
//! 项目级：.claude/skills/                    (ProjectSettings, 向上遍历至 home)
//! 附加目录：--add-dir 指定的路径下 .claude/skills/
//! ```
//! 
//! **加载协议**：只识别 `skill-name/SKILL.md` 目录格式。加载流程：
//! 1. `readdir` 扫描目录 → 仅保留 `is_directory()` 或 `is_symlink()` 的条目
//! 2. 在每个子目录中查找 `SKILL.md`，未找到则跳过
//! 3. `parse_frontmatter()` 解析 YAML 头部，提取 16 个 frontmatter 字段
//! 4. `create_skill_command()` 构造 `Command` 对象
//! 
//! **去重机制**：使用 `realpath()` 解析符号链接获得规范路径，避免通过符号链接或重叠父目录导致的重复加载。
//! 
//! ### 4. MCP Skills（动态发现）
//! 
//! 通过 `register_mcp_skill_builders()` 注册构建器，MCP Server 的 prompt 被转换为 `Command` 对象。
//! 
//! **安全边界**：MCP Skills 的 Prompt 内容**禁止执行内联 shell 命令**（因为远程内容不可信）。
//! 
//! ### 5. Legacy Commands（`/commands/` 目录）
//! 
//! 向后兼容的旧格式，同时支持 `SKILL.md` 目录格式和单 `.md` 文件格式。
//! 
//! ## Frontmatter 字段
//! 
//! ```yaml
//! ---
//! name: code-review                    # 显示名称（覆盖目录名）
//! description: 系统性代码审查           # 描述
//! when_to_use: "用户说审查代码、找 bug"  # AI 自动匹配依据
//! allowed-tools:                       # 工具白名单
//!   - Read
//!   - Grep
//!   - Glob
//! argument-hint: "<file-or-directory>" # 参数提示
//! arguments: [path]                    # 声明式参数名（用于 $ARGUMENTS 替换）
//! model: opus                          # 模型覆盖
//! effort: high                         # 努力级别
//! context: fork                        # 执行模式：inline（默认）| fork
//! agent: code-reviewer                 # 指定 Agent 定义文件
//! user-invocable: true                 # 用户是否可 / 调用
//! disable-model-invocation: false      # 禁止 AI 自主调用
//! version: "1.0"                       # 版本号
//! paths:                               # 条件激活的文件路径模式
//!   - "src/**/*.ts"
//! hooks:                               # Hook 配置
//!   PreToolUse:
//!     - command: ["echo", "checking"]
//! shell: ["bash"]                      # Shell 执行环境
//! ---
//! ```
//! 
//! ## 两条执行路径
//! 
//! ### Inline 模式（默认）
//! 
//! Skill 的 Prompt 内容被注入为 **UserMessage**，在主对话流中继续执行：
//! 1. 处理参数替换（`$ARGUMENTS`）和 shell 命令展开（`` !`...` ``）
//! 2. `${CLAUDE_SKILL_DIR}` 被替换为 Skill 所在目录的绝对路径
//! 3. `${CLAUDE_SESSION_ID}` 被替换为当前会话 ID
//! 4. 返回 `new_messages`（注入到对话流）+ `context_modifier`（修改权限上下文）
//! 
//! `context_modifier` 做了三件事：
//! - **工具白名单注入**：将 `allowed_tools` 合并到 `always_allow_rules.command`
//! - **模型切换**：处理模型覆盖，保留 `[1m]` 后缀以避免 200K 窗口截断
//! - **努力级别覆盖**：修改 `effort_value`
//! 
//! ### Fork 模式（`context: fork`）
//! 
//! Skill 在**独立子 Agent** 中执行：
//! 1. `prepare_forked_command_context()` 构建隔离的 Agent 定义和 Prompt
//! 2. `run_agent()` 启动子 Agent 循环，拥有独立的 token 预算
//! 3. 通过 `on_progress` 回调报告工具使用进度
//! 4. 结果通过 `extract_result_text()` 提取，子 Agent 的全部消息在提取后被释放
//! 5. 最终通过 `clear_invoked_skills_for_agent()` 清理状态
//! 
//! Fork 模式适用于需要强隔离的场景（如长时间运行的审查任务），避免污染主对话的上下文。
//! 
//! ## 权限模型：Safe Properties 白名单
//! 
//! 五层权限检查：
//! 
//! ```
//! 1. Deny 规则匹配（支持精确匹配和 prefix:* 通配符）
//!    ↓ 未命中
//! 2. 远程 canonical Skill 自动放行（FEATURE_FLAG + USER_TYPE === 'ant'）
//!    ↓ 未命中
//! 3. Allow 规则匹配
//!    ↓ 未命中
//! 4. Safe Properties 白名单检查（skill_has_only_safe_properties）
//!    ↓ 有非安全属性
//! 5. Ask 用户确认（附带精确匹配和前缀匹配两条建议规则）
//! ```
//! 
//! **Safe Properties** 是一个包含 30 个属性名的白名单。任何不在白名单中的**有意义的属性值**
//! （排除 `undefined`、`null`、空数组、空对象）都会触发权限请求。这是**正向安全**设计——
//! 未来新增的属性默认需要权限。
//! 
//! ## Prompt 预算：1% 上下文窗口的截断策略
//! 
//! Skill 列表注入 System Prompt 时有严格的字符预算：
//! - **预算计算**：`context_window_tokens × 4 chars/token × 1%`（约 8000 字符）
//! - **单条上限**：`MAX_LISTING_DESC_CHARS = 250` 字符（超出截断为 `…`）
//! - **Bundled Skills 不可截断**：它们始终保留完整描述，预算不足时只截断非 bundled 的
//! - **降级策略**：
//!   1. 尝试完整描述 → 超预算？
//!   2. Bundled 保留完整，非 bundled 均分剩余预算 → 每条描述低于 20 字符？
//!   3. 非 bundled 仅保留名称
//! 
//! ## 动态发现与条件激活
//! 
//! ### 基于文件路径的动态发现
//! 
//! `discover_skill_dirs_for_paths()` 在文件操作时触发：
//! 1. 从被操作的文件路径开始，**向上遍历**至 CWD（不包含 CWD 本身）
//! 2. 在每层查找 `.claude/skills/` 目录
//! 3. 使用 `realpath` 去重，`git check-ignore` 过滤 gitignored 目录
//! 4. 按路径深度排序（**深层优先**），更接近文件的 Skill 优先级更高
//! 
//! ### 条件激活（paths frontmatter）
//! 
//! 带有 `paths` 模式的 Skill 在加载时不会立即可用，而是存入 `conditional_skills` Map。
//! 当被操作的文件路径匹配某个 Skill 的 paths 模式时（使用 `ignore` 库做 gitignore 风格匹配），
//! 该 Skill 才被**激活**——从 `conditional_skills` 移入 `dynamic_skills`。
//! 
//! 这意味着一个只在 `*.test.ts` 上激活的测试 Skill，平时完全不可见，只有当 AI 读取或编辑测试文件时才会出现。
//! 
//! ## 使用频率排名
//! 
//! `record_skill_usage()` 使用指数衰减算法计算 Skill 排名分数：
//! 
//! ```
//! score = usage_count × max(0.5^(days_since_use / 7), 0.1)
//! ```
//! 
//! - **7 天半衰期**：一周前的使用权重减半
//! - **最低 0.1 保底**：避免老但高频使用的 Skill 完全沉底
//! - **60 秒去抖**：同一 Skill 在 1 分钟内的多次调用只计一次，减少文件 I/O
//! 
//! ## 完整生命周期
//! 
//! ```
//! 磁盘 SKILL.md
//!   ↓ parse_frontmatter()
//!   ↓ parse_skill_frontmatter_fields() → 16 个字段
//!   ↓ create_skill_command() → Command 对象
//!   ↓ 去重（realpath + seen_file_ids）
//!   ↓ 条件 Skill → conditional_skills Map（等待路径匹配激活）
//!   ↓ get_skill_dir_commands() memoize 缓存
//!   ↓ get_all_commands() 合并 local + MCP
//!   ↓ format_commands_within_budget() → 截断后的 Skill 列表注入 System Prompt
//!   ↓ AI 选择匹配的 Skill
//!   ↓ validate_input() → 名称校验 + 存在性检查
//!   ↓ check_permissions() → 五层权限检查
//!   ↓ execute() → inline 或 fork 执行
//!   ↓ context_modifier() → 注入 allowed_tools + model + effort
//!   ↓ record_skill_usage() → 更新使用频率排名
//! ```
//! 
//! ## 模块结构
//! 
//! ```
//! skills/
//! ├── mod.rs              # 模块入口
//! ├── types.rs            # 类型定义和 Frontmatter 解析
//! ├── loader.rs           # Skill 加载器（磁盘/内置/MCP）
//! ├── executor.rs         # 执行引擎（Inline/Fork 模式）
//! └── permissions.rs      # 权限模型（白名单/安全属性）
//! ```

pub mod types;
pub mod loader;
pub mod executor;
pub mod permissions;

pub use types::{
    SkillCommand, SkillFrontmatter, SkillSource, SkillLoadSource,
    ModelOverride, EffortLevel, ExecutionContext, HookConfig,
    SkillUsage, ConditionalSkills, SAFE_SKILL_PROPERTIES,
    parse_frontmatter,
};
pub use loader::SkillLoader;
pub use executor::{SkillExecutor, SkillExecutionResult, SkillExecutionError, ContextModifier};
pub use permissions::{
    SkillPermissionChecker, PermissionCheckResult, PermissionRule, RuleType, RuleSource,
};
