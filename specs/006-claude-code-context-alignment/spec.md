# Feature Specification: Claude Code Context Alignment

**Feature Branch**: `006-claude-code-context-alignment`
**Created**: 2026-04-19
**Updated**: 2026-04-21
**Status**: Draft
**Input**: User description: "位于 references/claude-code/src/context.ts 的功能，需要对齐到当前项目。"

## Summary

将 Claude Code 的上下文注入功能对齐到 devil-agent。上下文在每次对话开始时被预处理并添加到对话中，包括 Git 状态、当前日期和 Memory 文件（CLAUDE.md）内容。

## Architecture

### 2.1 Token Budget System

系统定义三层 Token 预算体系，用于上下文总量控制：

| 预算级别 | 最大 Token 数 | 适用场景 | 优先级 |
|---------|--------------|---------|--------|
| **Compact** | 50,000 | 资源受限环境、快速响应 | 最低 |
| **Standard** | 100,000 | 常规开发任务 | 默认 |
| **Extended** | 200,000 | 复杂分析、长上下文任务 | 最高 |

```rust
pub enum TokenBudget {
    Compact,    // 50K tokens
    Standard,   // 100K tokens (默认)
    Extended,   // 200K tokens
}

pub struct ContextConfig {
    pub budget: TokenBudget,
    pub max_context_tokens: usize,
}

impl Default for ContextConfig {
    fn default() -> Self {
        Self {
            budget: TokenBudget::Standard,
            max_context_tokens: 100_000,
        }
    }
}

impl TokenBudget {
    pub fn max_tokens(&self) -> usize {
        match self {
            TokenBudget::Compact => 50_000,
            TokenBudget::Standard => 100_000,
            TokenBudget::Extended => 200_000,
        }
    }
}
```

### 2.2 Context Injection Priority

上下文注入内容按以下优先级排序，高优先级内容在预算紧张时优先保留：

| 优先级 | 内容类型 | 说明 |
|-------|---------|------|
| P0 | System Prompt | 系统级指令，必须保留 |
| P1 | Memory Files (CLAUDE.md) | 用户定义的 AI 指导内容 |
| P1 | Current Date | 当前日期和时间背景 |
| P2 | Git Status | 分支、主分支、状态、最近提交 |
| P3 | Tool Results (Recent) | 最近工具执行结果（可压缩） |
| P3 | Historical Messages | 历史对话记录（可压缩） |
| P4 | Cache Breaker | 调试用的缓存清除标记 |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum InjectionPriority {
    P0_SystemPrompt = 0,
    P1_MemoryFiles = 10,
    P1_CurrentDate = 11,
    P2_GitStatus = 20,
    P3_ToolResults = 30,
    P3_HistoricalMessages = 31,
    P4_CacheBreaker = 40,
}

pub struct InjectedContent {
    pub priority: InjectionPriority,
    pub content: String,
    pub token_count: usize,
    pub is_compressible: bool,
}
```

### 2.3 Seven-Step Compression Pipeline

七步压缩管线在每次上下文更新时执行，确保总量不超过 Token 预算：

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Seven-Step Compression Pipeline                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌──────────────┐    ┌──────────┐    ┌───────────────┐    ┌──────────────┐  │
│  │ Step 1       │───▶│ Step 2   │───▶│ Step 3        │───▶│ Step 4       │  │
│  │ Tool Result  │    │ Snip     │    │ MicroCompact  │    │ Collapse     │  │
│  │ Budget       │    │          │    │               │    │              │  │
│  └──────────────┘    └──────────┘    └───────────────┘    └──────────────┘  │
│         │                │                  │                   │          │
│         ▼                ▼                  ▼                   ▼          │
│  ┌──────────────┐    ┌──────────┐    ┌───────────────┐    ┌──────────────┐  │
│  │ 工具结果     │    │ 裁剪标记  │    │ 缓存过期      │    │ 主动重构      │  │
│  │ Token 限制   │    │ 清除      │    │ 清理          │    │              │  │
│  └──────────────┘    └──────────┘    └───────────────┘    └──────────────┘  │
│                                                                              │
│  ┌──────────────┐    ┌──────────┐    ┌───────────────┐                      │
│  │ Step 5       │───▶│ Step 6   │───▶│ Step 7        │                      │
│  │ AutoCompact  │    │ Hard     │    │ Block         │                      │
│  │              │    │ Truncate │    │               │                      │
│  └──────────────┘    └──────────┘    └───────────────┘                      │
│         │                │                  │                               │
│         ▼                ▼                  ▼                               │
│  ┌──────────────┐    ┌──────────┐    ┌───────────────┐                     │
│  │ 摘要生成      │    │ 硬截断    │    │ 阻断检查       │                     │
│  └──────────────┘    └──────────┘    └───────────────┘                     │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Step 1: Tool Result Budget

工具结果 Token 限制，控制单次工具调用的结果量：

```rust
pub struct ToolResultBudget {
    pub max_tokens_per_result: usize,  // 单个工具结果最大 Token
    pub max_total_results: usize,       // 上下文中保留的最大结果数
}

impl Default for ToolResultBudget {
    fn default() -> Self {
        Self {
            max_tokens_per_result: 4_000,   // 单结果 4K tokens
            max_total_results: 10,           // 最多保留 10 个结果
        }
    }
}

pub fn apply_tool_result_budget(messages: &mut Vec<Message>, budget: &ToolResultBudget) {
    // 1. 对每个工具结果按时间排序（从新到旧）
    // 2. 保留最近 N 个结果
    // 3. 对单个超长结果进行截断
}
```

**规则**：
- 单个工具结果超过 `max_tokens_per_result` 时，截断并添加 `[truncated]` 标记
- 保留最近 `max_total_results` 个工具结果
- 超出部分放入压缩缓冲区待后续处理

#### Step 2: Snip (裁剪标记清除)

清除文本中的裁剪标记和占位符，精简内容：

```rust
pub struct SnipConfig {
    pub remove_omitted_markers: bool,  // 清除 "[... X lines omitted]" 类标记
    pub trim_whitespace: bool,         // 清除首尾空白
    pub normalize_newlines: bool,      // 规范换行符
}

impl Default for SnipConfig {
    fn default() -> Self {
        Self {
            remove_omitted_markers: true,
            trim_whitespace: true,
            normalize_newlines: true,
        }
    }
}

pub fn snip(content: &str, config: &SnipConfig) -> String {
    let mut result = content.to_string();
    
    if config.remove_omitted_markers {
        // 移除省略标记: [... X lines omitted], [X lines hidden], etc.
        result = result.replace_regex(r"\[?\.\.\.\s*\d+\s*lines?\s*(omitted|hidden|redacted)?\]?", "");
    }
    
    if config.trim_whitespace {
        result = result.trim().to_string();
    }
    
    if config.normalize_newlines {
        // 将多个连续换行压缩为最多两个
        result = result.replace_regex(r"\n{3,}", "\n\n");
    }
    
    result
}
```

**规则**：
- 移除 `[... X lines omitted]` 类标记
- 合并多个连续空行
- 清除行首行尾空白

#### Step 3: MicroCompact (缓存过期清理)

清理过期的缓存信息，释放 Token 空间：

```rust
pub struct CacheEntry {
    pub key: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub ttl_seconds: u64,
    pub access_count: u32,
}

pub struct MicroCompactConfig {
    pub cache_ttl_seconds: u64,        // 缓存 TTL，默认 1 小时
    pub min_access_count: u32,         // 最小访问次数，低于此值可清理
    pub max_cache_entries: usize,      // 最大缓存条目数
}

impl Default for MicroCompactConfig {
    fn default() -> Self {
        Self {
            cache_ttl_seconds: 3600,      // 1 小时
            min_access_count: 2,           // 访问少于 2 次可清理
            max_cache_entries: 50,
        }
    }
}

pub fn micro_compact(cache: &mut HashMap<String, CacheEntry>, config: &MicroCompactConfig) {
    let now = Utc::now();
    
    cache.retain(|_, entry| {
        // 保留条件（满足任一）：
        // 1. 未过期且访问次数 >= min_access_count
        // 2. 在 max_cache_entries 限制内且最近访问
        let age = now.signed_duration_since(entry.created_at).num_seconds();
        let is_expired = age > config.cache_ttl_seconds as i64;
        let is_rare = entry.access_count < config.min_access_count;
        
        !is_expired && !(is_rare && cache.len() > config.max_cache_entries / 2)
    });
}
```

**规则**：
- 清除超过 TTL 的缓存条目
- 清理访问频率低的内容
- 优先保留高频访问的缓存

#### Step 4: Collapse (主动重构)

对相似或重复内容进行主动重构，消除冗余：

```rust
pub struct CollapseConfig {
    pub similarity_threshold: f32,     // 相似度阈值，0.0-1.0
    pub max_history_turns: usize,      // 保留的最大历史轮次
}

impl Default for CollapseConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,    // 85% 相似度触发合并
            max_history_turns: 20,         // 保留最近 20 轮
        }
    }
}

pub fn collapse(messages: &mut Vec<Message>, config: &CollapseConfig) {
    // 1. 识别相似消息对
    // 2. 合并重复的概念解释
    // 3. 压缩连续的单轮对话
    // 4. 保留最近 max_history_turns 轮
    
    messages.truncate(config.max_history_turns * 2); // 每轮包含 user+assistant
}
```

**规则**：
- 合并高度相似的连续消息
- 保留最近 `max_history_turns` 轮对话
- 超出部分进行摘要压缩

#### Step 5: AutoCompact (摘要生成)

对压缩缓冲区内容生成摘要，保留关键信息：

```rust
pub struct AutoCompactConfig {
    pub compression_ratio: f32,        // 压缩比，如 0.3 表示压缩到 30%
    pub preserve_key_info: bool,       // 保留关键信息（数字、名称、路径）
    pub summary_style: SummaryStyle,
}

#[derive(Debug, Clone, Copy)]
pub enum SummaryStyle {
    Concise,    // 简洁摘要
    Detailed,   // 详细摘要
    Technical,  // 技术摘要
}

impl Default for AutoCompactConfig {
    fn default() -> Self {
        Self {
            compression_ratio: 0.3,        // 压缩到 30%
            preserve_key_info: true,
            summary_style: SummaryStyle::Concise,
        }
    }
}

pub async fn auto_compact(
    buffer: &str,
    budget: usize,
    config: &AutoCompactConfig,
) -> Result<String> {
    // 调用 LLM 生成摘要
    let prompt = format!(
        "压缩以下内容，保留关键信息和语义，输出不超过 {} tokens:\n\n{}",
        budget,
        buffer
    );
    
    // 调用 LLM 生成摘要...
    Ok(summary)
}
```

**规则**：
- 将超容内容压缩到预算范围内
- 优先保留关键信息（文件路径、函数名、数字）
- 支持多种摘要风格

#### Step 6: Hard Truncate (硬截断)

作为最后手段，直接截断超出预算的内容：

```rust
pub struct HardTruncateConfig {
    pub emergency_threshold: f32,      // 触发紧急截断的阈值
    pub preserve_tail_tokens: usize,   // 保留末尾 token 数
}

impl Default for HardTruncateConfig {
    fn default() -> Self {
        Self {
            emergency_threshold: 0.95,   // 95% 容量时触发
            preserve_tail_tokens: 1000,  // 保留最后 1K tokens
        }
    }
}

pub fn hard_truncate(content: &str, max_tokens: usize) -> String {
    // 1. 按 token 计数
    // 2. 从末尾向前截断
    // 3. 添加截断标记
    
    let truncated = &content[..max_tokens];
    format!(
        "{}\n\n[Context truncated. {} tokens removed]",
        truncated,
        content.len() - truncated.len()
    )
}
```

**规则**：
- 仅在其他压缩手段无法满足预算时触发
- 从末尾向前截断，保留最新内容
- 添加明确的截断标记

#### Step 7: Block (阻断检查)

最终检查，确认上下文符合所有约束：

```rust
pub struct BlockCheckResult {
    pub passed: bool,
    pub issues: Vec<ContextIssue>,
}

pub enum ContextIssue {
    TokenLimitExceeded { actual: usize, limit: usize },
    PriorityViolation { content_type: String },
    ProhibitedContent { pattern: String },
}

pub fn block_check(context: &Context, config: &ContextConfig) -> BlockCheckResult {
    let mut issues = Vec::new();
    
    // 1. Token 总量检查
    let actual_tokens = count_tokens(&context.to_string());
    if actual_tokens > config.max_context_tokens {
        issues.push(ContextIssue::TokenLimitExceeded {
            actual: actual_tokens,
            limit: config.max_context_tokens,
        });
    }
    
    // 2. 优先级顺序检查
    // 确保高优先级内容未被意外压缩
    
    // 3. 禁止内容检查
    // 检查是否包含敏感或不当内容
    
    BlockCheckResult {
        passed: issues.is_empty(),
        issues,
    }
}
```

**规则**：
- Token 总量不得超过预算
- 高优先级内容不得被压缩或截断
- 不得包含禁止内容

### 2.4 External Knowledge Base Integration

外部知识库集成为上下文系统提供扩展能力：

```rust
pub struct KnowledgeBaseConfig {
    pub enabled: bool,
    pub provider: KnowledgeProvider,
    pub max_tokens: usize,           // 从知识库获取的最大 token 数
    pub relevance_threshold: f32,    // 相关性阈值
}

#[derive(Debug, Clone)]
pub enum KnowledgeProvider {
    None,
    Local { path: String },          // 本地知识库目录
    Web { search_enabled: bool },    // Web 搜索
    // 可扩展: VectorDB, GraphDB, etc.
}

pub struct KnowledgeQuery {
    pub query: String,
    pub top_k: usize,
    pub filters: Option<HashMap<String, String>>,
}

pub async fn query_knowledge_base(
    config: &KnowledgeBaseConfig,
    query: &KnowledgeQuery,
) -> Result<Vec<KnowledgeEntry>> {
    match &config.provider {
        KnowledgeProvider::None => Ok(vec![]),
        KnowledgeProvider::Local { path } => {
            // 从本地目录检索相关文件
            let entries = search_local_knowledge(path, query).await?;
            Ok(entries.into_iter().take(query.top_k).collect())
        }
        KnowledgeProvider::Web { search_enabled } if *search_enabled => {
            // 执行 Web 搜索
            let entries = web_search(query).await?;
            Ok(entries.into_iter().take(query.top_k).collect())
        }
        _ => Ok(vec![]),
    }
}

pub struct KnowledgeEntry {
    pub title: String,
    pub content: String,
    pub source: String,
    pub relevance_score: f32,
}
```

**集成方式**：
- **本地知识库**: 扫描指定目录的 Markdown/文本文件
- **Web 搜索**: 对知识密集型查询启用
- **向量数据库**: (规划中) 语义检索

## User Scenarios & Testing

### User Story 1 - Git Status Context Injection (Priority: P2)

AI Agent 自动收集并注入当前 Git 仓库状态到系统上下文，包括分支名称、主分支、状态和最近提交。

**Why this priority**: Git 状态是开发者工作流的基础信息，帮助 AI 了解当前代码库状态。

**Independent Test**: 执行 `git status` 和 `git log` 命令，验证输出包含正确的分支信息。

**Acceptance Scenarios**:

1. **Given** 当前目录是 Git 仓库，**When** AI Agent 启动对话，**Then** Git 状态信息被包含在系统上下文中
2. **Given** 当前目录不是 Git 仓库，**When** AI Agent 启动对话，**Then** Git 状态信息被跳过
3. **Given** Git status 输出超过 2000 字符，**When** AI Agent 启动对话，**Then** 输出被截断并显示提示

---

### User Story 2 - Current Date Context (Priority: P1)

AI Agent 自动注入当前日期到用户上下文，帮助 AI 了解时间背景。

**Why this priority**: 日期信息对于理解代码修改时间、项目进度和任务规划至关重要。

**Independent Test**: 验证系统上下文包含格式化的当前日期。

**Acceptance Scenarios**:

1. **Given** AI Agent 正常运行，**When** 对话开始，**Then** 当前日期被包含在上下文中

---

### User Story 3 - Memory Files (CLAUDE.md) Context (Priority: P1)

AI Agent 自动发现并加载项目中的 CLAUDE.md 文件，将内容注入到上下文。

**Why this priority**: CLAUDE.md 文件允许开发者为 AI 提供项目特定的指导和上下文。

**Independent Test**: 创建测试用的 CLAUDE.md 文件，验证其内容被正确加载。

**Acceptance Scenarios**:

1. **Given** 项目根目录存在 CLAUDE.md，**When** AI Agent 启动对话，**Then** 文件内容被包含在上下文中
2. **Given** 项目不存在 CLAUDE.md，**When** AI Agent 启动对话，**Then** Memory 文件上下文为空
3. **Given** 环境变量 CLAUDE_CODE_DISABLE_CLAUDE_MDS=1，**When** AI Agent 启动对话，**Then** Memory 文件被禁用

---

### User Story 4 - Cache Breaker / System Prompt Injection (Priority: P4)

支持系统提示注入功能，用于强制刷新 AI 的缓存上下文。

**Why this priority**: 用于调试和特殊场景，需要绕过正常缓存机制。

**Independent Test**: 设置 system prompt injection，验证缓存被正确清除。

**Acceptance Scenarios**:

1. **Given** system prompt injection 被设置，**When** 对话开始，**Then** 缓存被清除并应用新的 injection

---

### User Story 5 - Bare Mode Support (Priority: P3)

在 --bare 模式下，AI Agent 跳过自动发现行为，但仍然处理显式添加的目录。

**Why this priority**: 与 Claude Code 行为保持一致，支持高级用户控制。

**Independent Test**: 使用 --bare 模式启动，验证行为符合预期。

**Acceptance Scenarios**:

1. **Given** 使用 --bare 模式且无显式添加目录，**When** AI Agent 启动，**Then** 跳过 Memory 文件自动发现
2. **Given** 使用 --bare 模式且有显式添加目录，**When** AI Agent 启动，**Then** 仍然处理显式添加的目录

---

### User Story 6 - Token Budget Enforcement (Priority: P0)

系统强制执行 Token 预算限制，确保上下文不超过最大容量。

**Why this priority**: Token 预算控制是系统稳定性和性能的基础保障。

**Independent Test**: 创建大量历史消息，验证压缩管线正确执行。

**Acceptance Scenarios**:

1. **Given** 上下文内容超过 Token 预算，**When** 消息发送，**Then** 七步压缩管线自动执行
2. **Given** Token 预算设为 50K，**When** 上下文达到 50K tokens，**Then** 高优先级内容被保留，低优先级被压缩
3. **Given** 极端情况下压缩失败，**When** 上下文仍超预算，**Then** 硬截断执行并提示用户

---

### User Story 7 - External Knowledge Base Query (Priority: P2)

系统支持查询外部知识库，增强上下文信息。

**Why this priority**: 外部知识库为复杂问题提供额外参考信息。

**Independent Test**: 配置本地知识库路径，验证相关文档被正确检索。

**Acceptance Scenarios**:

1. **Given** 配置了本地知识库路径，**When** 用户询问相关主题，**Then** 知识库内容被检索并注入上下文
2. **Given** 知识库条目超过 max_tokens 限制，**When** 检索结果返回，**Then** 结果被截断或摘要

---

## Edge Cases

- Git 命令执行失败时如何处理？
- CLAUDE.md 文件不存在或为空时如何处理？
- Git status 输出被截断后，用户如何获取完整信息？
- Token 预算耗尽时，如何保证 P0/P1 内容的完整性？
- 外部知识库无相关结果时，如何处理？
- 压缩管线中某一步骤失败时的回退策略？

## Requirements

### Functional Requirements

- **FR-001**: 系统必须在对话开始时收集 Git 状态信息（分支、主分支、状态、最近提交）
- **FR-002**: 系统必须在上下文中包含当前日期，格式为 ISO 日期
- **FR-003**: 系统必须自动发现并加载项目中的 CLAUDE.md 文件
- **FR-004**: 系统必须支持通过环境变量 CLAUDE_CODE_DISABLE_CLAUDE_MDS 禁用 Memory 文件
- **FR-005**: Git status 输出超过 2000 字符时必须截断，并提供替代方案提示
- **FR-006**: 系统上下文必须被缓存以提高性能
- **FR-007**: 系统必须支持缓存清除机制（system prompt injection）
- **FR-008**: 在 bare 模式下，系统必须跳过自动发现但处理显式添加的目录
- **FR-009**: 非 Git 目录中启动时，系统必须优雅地跳过 Git 状态收集
- **FR-010**: 系统必须支持三层 Token 预算体系（50K/100K/200K）
- **FR-011**: 七步压缩管线必须在上下文超预算时自动执行
- **FR-012**: 上下文注入必须遵循优先级顺序（P0>P1>P2>P3>P4）
- **FR-013**: 硬截断作为最后手段必须保留截断标记
- **FR-014**: 系统必须支持外部知识库集成（本地目录/Web 搜索）
- **FR-015**: 阻断检查必须确保所有约束条件被满足

### Non-Functional Requirements

- **NFR-001**: 压缩管线执行时间不得超过 500ms
- **NFR-002**: 上下文初始化时间不得超过 200ms
- **NFR-003**: Token 计数误差不得超过 ±5%

### Key Entities

- **SystemContext**: 系统级上下文，包含 Git 状态和缓存断路器
- **UserContext**: 用户级上下文，包含 Memory 文件内容和当前日期
- **GitStatus**: Git 仓库状态（分支、主分支、状态、最近提交）
- **MemoryFiles**: CLAUDE.md 文件的集合及其内容
- **TokenBudget**: Token 预算配置
- **CompressionPipeline**: 七步压缩管线
- **KnowledgeBase**: 外部知识库接口

## Interface Definitions

### ContextManager Interface

```rust
#[async_trait]
pub trait ContextManager: Send + Sync {
    async fn initialize(&self, session_id: &str) -> Result<Context>;
    async fn inject(&self, session_id: &str, content: InjectedContent) -> Result<()>;
    async fn compress(&self, session_id: &str) -> Result<CompressionReport>;
    async fn query_knowledge(&self, query: &str) -> Result<Vec<KnowledgeEntry>>;
}
```

### CompressionPipeline Interface

```rust
#[async_trait]
pub trait CompressionPipeline: Send + Sync {
    fn step1_tool_result_budget(&self, messages: &mut Vec<Message>);
    fn step2_snip(&self, content: &str) -> String;
    fn step3_micro_compact(&self, cache: &mut HashMap<String, CacheEntry>);
    fn step4_collapse(&self, messages: &mut Vec<Message>);
    async fn step5_auto_compact(&self, buffer: &str, budget: usize) -> Result<String>;
    fn step6_hard_truncate(&self, content: &str, max_tokens: usize) -> String;
    fn step7_block_check(&self, context: &Context) -> BlockCheckResult;
    
    async fn run(&self, context: &mut Context, config: &ContextConfig) -> Result<CompressionReport>;
}
```

## Success Criteria

### Measurable Outcomes

- **SC-001**: Git 仓库中启动对话时，上下文中包含正确的分支名称
- **SC-002**: 所有对话中都包含当前日期信息
- **SC-003**: 存在 CLAUDE.md 的项目中，内容被正确加载到上下文
- **SC-004**: Git status 超过 2000 字符时自动截断
- **SC-005**: 上下文缓存正常工作，重复调用不触发额外文件系统/Git 操作
- **SC-006**: 设置 injection 后缓存被正确清除
- **SC-007**: Token 预算强制执行，上下文中不超过配置的限制
- **SC-008**: 压缩管线正确执行，各步骤按顺序处理
- **SC-009**: 优先级顺序正确，高优先级内容在预算紧张时被保留
- **SC-010**: 外部知识库查询返回相关结果

## Assumptions

- 用户主要在 Git 仓库中使用 AI Agent
- CLAUDE.md 文件遵循标准格式（Markdown）
- 日期格式使用 ISO 8601 标准（YYYY-MM-DD）
- Git 命令在 PATH 中可用
- Token 计数使用准确的 tiktoken 或等效库
- 外部知识库文件为 UTF-8 编码

## Dependencies

- **spec-003**: 工具系统（工具结果注入上下文）
- **spec-004**: 权限系统（上下文访问权限）
- **spec-007**: 记忆系统（MEMORY.md 索引）

## References

- Claude Code context.ts: `references/claude-code/src/context.ts`
- Token Budget Specification: 50K/100K/200K tiers
- Compression Pipeline: Seven-step model
