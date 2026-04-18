//! 四级压缩策略实现
//!
//! 从低成本到高成本依次递进：
//! 1. Snip - 标记清除
//! 2. MicroCompact - 时间触发
//! 3. Collapse - 主动重构
//! 4. AutoCompact - 对话摘要

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 可压缩的工具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressibleToolType {
    Read,
    Bash,
    Grep,
    Glob,
    WebSearch,
    WebFetch,
    Edit,
    Write,
}

impl CompressibleToolType {
    /// 判断工具类型是否可压缩
    pub fn is_compressible(tool_name: &str) -> bool {
        let tool_lower = tool_name.to_lowercase();
        matches!(
            tool_lower.as_str(),
            "read" | "bash" | "grep" | "glob" | "websearch" | "webfetch" | "edit" | "write"
        )
    }

    /// 从工具名称解析
    pub fn from_tool_name(tool_name: &str) -> Option<Self> {
        match tool_name.to_lowercase().as_str() {
            "read" => Some(Self::Read),
            "bash" => Some(Self::Bash),
            "grep" => Some(Self::Grep),
            "glob" => Some(Self::Glob),
            "websearch" => Some(Self::WebSearch),
            "webfetch" => Some(Self::WebFetch),
            "edit" => Some(Self::Edit),
            "write" => Some(Self::Write),
            _ => None,
        }
    }
}

/// Snip 标记文本
pub const SNIP_MARKER_TEXT: &str = "[Old tool result content cleared]";

/// Level 1: Snip（裁剪）配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipConfig {
    /// 要清除的消息 UUID 列表
    pub message_uuids: Vec<String>,
    /// 是否保留消息结构（推荐 true）
    pub preserve_structure: bool,
}

impl Default for SnipConfig {
    fn default() -> Self {
        Self {
            message_uuids: Vec::new(),
            preserve_structure: true,
        }
    }
}

/// Snip 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnipResult {
    /// 已清除的消息数量
    pub cleared_count: u32,
    /// 估计释放的令牌数
    pub estimated_tokens_freed: u32,
}

/// Level 1: Snip（裁剪）实现
///
/// 最轻量的压缩手段，不调用 LLM
/// 直接将旧的工具结果内容替换为标记文本
pub struct SnipCompression;

impl SnipCompression {
    /// 执行 Snip 压缩
    ///
    /// # 参数
    ///
    /// * `config` - Snip 配置
    ///
    /// # 返回
    ///
    /// Snip 结果
    pub fn execute(config: &SnipConfig) -> SnipResult {
        // 实际实现会遍历消息，将工具结果替换为 SNIP_MARKER_TEXT
        SnipResult {
            cleared_count: config.message_uuids.len() as u32,
            // 估计值，实际应该根据内容计算
            estimated_tokens_freed: config.message_uuids.len() as u32 * 500,
        }
    }

    /// 为什么保留消息结构而不是直接删除？
    ///
    /// 因为删除消息会破坏消息链的连续性——后续消息可能引用了前面的工具调用 ID。
    /// 标记文本既释放了空间，又保持了消息结构完整。
    pub fn preserve_structure_rationale() -> &'static str {
        "Preserving message structure maintains tool call ID references and conversation continuity"
    }
}

/// Level 2: MicroCompact（微压缩）配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCompactConfig {
    /// 保留最近的 N 个工具结果
    pub keep_recent: u32,
    /// 时间阈值（距离上次助手消息的时间）
    pub time_threshold_secs: u64,
    /// 是否启用
    pub enabled: bool,
}

impl Default for MicroCompactConfig {
    fn default() -> Self {
        Self {
            // 保留最近 5-10 个工具结果
            keep_recent: 5,
            // 默认 60 分钟阈值
            time_threshold_secs: 3600,
            enabled: true,
        }
    }
}

/// MicroCompact 触发评估结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroCompactEvaluation {
    /// 是否应该触发
    pub should_trigger: bool,
    /// 距离上次助手消息的时间
    pub time_since_last_assistant: Duration,
    /// 触发原因
    pub reason: Option<String>,
}

/// Level 2: MicroCompact（微压缩）实现
///
/// 基于时间触发的大规模工具结果清理
/// 当服务端缓存过期时，主动清除旧的工具结果
pub struct MicroCompactCompression {
    config: MicroCompactConfig,
    last_assistant_message_time: Option<Instant>,
}

impl MicroCompactCompression {
    pub fn new(config: MicroCompactConfig) -> Self {
        Self {
            config,
            last_assistant_message_time: None,
        }
    }

    /// 更新上次助手消息时间
    pub fn update_last_assistant_time(&mut self, time: Instant) {
        self.last_assistant_message_time = Some(time);
    }

    /// 评估是否应该触发微压缩
    ///
    /// # 为什么与缓存过期有关？
    ///
    /// Claude 的 API 支持提示缓存（Prompt Caching）——如果连续请求的前缀相同，
    /// 缓存命中的部分可以大幅降低成本和延迟。但随着时间推移，缓存会过期。
    /// 当缓存过期时，无论如何都需要重新发送完整内容。此时保留旧的工具结果只是徒增负载。
    pub fn evaluate(&self) -> MicroCompactEvaluation {
        let Some(last_time) = self.last_assistant_message_time else {
            return MicroCompactEvaluation {
                should_trigger: false,
                time_since_last_assistant: Duration::MAX,
                reason: Some("No previous assistant message".to_string()),
            };
        };

        let elapsed = last_time.elapsed();
        let should_trigger =
            self.config.enabled && elapsed >= Duration::from_secs(self.config.time_threshold_secs);

        MicroCompactEvaluation {
            should_trigger,
            time_since_last_assistant: elapsed,
            reason: if should_trigger {
                Some(format!(
                    "Cache expired: {} seconds since last assistant message (threshold: {}s)",
                    elapsed.as_secs(),
                    self.config.time_threshold_secs
                ))
            } else {
                Some(format!(
                    "Cache still valid: {} seconds since last assistant message",
                    elapsed.as_secs()
                ))
            },
        }
    }

    /// 执行微压缩
    ///
    /// 保留最近 N 个可压缩工具结果，清除其余的
    pub fn execute(&self, tool_results: &[String]) -> (Vec<String>, u32) {
        let keep_count = self.config.keep_recent as usize;

        if tool_results.len() <= keep_count {
            return (tool_results.to_vec(), 0);
        }

        // 保留最近的 N 个，清除前面的
        let clear_count = tool_results.len() - keep_count;
        let mut result = Vec::with_capacity(tool_results.len());

        // 清除旧的内容
        for _ in 0..clear_count {
            result.push(SNIP_MARKER_TEXT.to_string());
        }

        // 保留最近的内容
        result.extend_from_slice(&tool_results[clear_count..]);

        (result, clear_count as u32)
    }

    /// 基于缓存编辑的无损优化
    ///
    /// 通过 API 层面的 cache_edits 机制删除工具结果而不破坏缓存前缀
    pub fn supports_cache_edits() -> bool {
        // 实际实现取决于 API 支持
        true
    }
}

/// Level 3: Collapse（折叠）配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseConfig {
    /// 触发阈值（0.0-1.0）
    pub trigger_threshold: f32,
    /// 阻止 spawn 阈值
    pub block_spawn_threshold: f32,
    /// 是否抑制自动压缩
    pub suppress_auto_compact: bool,
}

impl Default for CollapseConfig {
    fn default() -> Self {
        Self {
            // 90% 利用率开始提交
            trigger_threshold: 0.90,
            // 95% 阻止新 spawn
            block_spawn_threshold: 0.95,
            // 抑制自动压缩（避免竞争）
            suppress_auto_compact: true,
        }
    }
}

/// Collapse 核心权衡
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseTradeoff {
    /// 触发时机
    pub trigger_timing: String,
    /// 压缩粒度
    pub granularity: String,
    /// 信息保留
    pub retention: String,
    /// 与 Fork 的关系
    pub fork_behavior: String,
}

impl CollapseTradeoff {
    pub fn new() -> Self {
        Self {
            trigger_timing: "90% utilization (proactive)".to_string(),
            granularity: "Selective message group reconstruction".to_string(),
            retention: "More original details preserved".to_string(),
            fork_behavior: "Blocks new spawns at 95%".to_string(),
        }
    }
}

impl Default for CollapseTradeoff {
    fn default() -> Self {
        Self::new()
    }
}

/// Level 3: Collapse（折叠）实现
///
/// 上下文重构级压缩
/// 在设计哲学上与 AutoCompact 不同："在空间压力出现之前就主动重构"
pub struct CollapseCompression {
    config: CollapseConfig,
}

impl CollapseCompression {
    pub fn new(config: CollapseConfig) -> Self {
        Self { config }
    }

    /// 判断是否应该触发 Collapse
    pub fn should_trigger(&self, usage_ratio: f32) -> bool {
        usage_ratio >= self.config.trigger_threshold
    }

    /// 判断是否应该阻止新 spawn
    pub fn should_block_spawn(&self, usage_ratio: f32) -> bool {
        usage_ratio >= self.config.block_spawn_threshold
    }

    /// Collapse vs AutoCompact 的关键区别
    pub fn compare_with_auto_compact() -> CollapseTradeoff {
        CollapseTradeoff::new()
    }

    /// Collapse 模式会抑制自动压缩的触发
    ///
    /// 因为两者在 93% 的临界点会产生竞争
    /// Collapse 作为更精细的上下文管理系统，拥有更高的优先级
    pub fn suppresses_auto_compact(&self) -> bool {
        self.config.suppress_auto_compact
    }
}

/// Level 4: AutoCompact（自动压缩）提示词模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompactPromptTemplate {
    /// BASE - 全量对话摘要
    Base,
    /// PARTIAL - 仅摘要最近消息（from 方向）
    Partial,
    /// PARTIAL_UP_TO - 摘要指定消息之前的上下文（up_to 方向）
    PartialUpTo,
}

/// 压缩提示词模板内容
pub struct CompactPrompts;

impl CompactPrompts {
    /// BASE_COMPACT_PROMPT - 全量对话摘要
    ///
    /// 场景：从对话开始到当前的所有消息
    /// 适用：常规自动压缩
    pub const BASE: &'static str = r#"Please summarize this conversation. Provide your response in the following format:

<analysis>
[Your analysis and thinking process - this will be discarded]
</analysis>

<summary>
## Goals and Intentions
[What the user is trying to achieve]

## Key Decisions and Changes
[Important decisions made, changes to approach]

## Current State
[Where things stand now]

## Files Modified
[List of files changed and why]

## Unresolved Questions
[Open questions or pending decisions]

## Tool Usage Summary
[What tools were used and why]

## Next Steps
[What should happen next]

## Important Context to Preserve
[Any context that must be preserved for future turns]

## Additional Notes
[Any other relevant information]
</summary>

IMPORTANT: Do NOT use any tools (Read, Bash, etc.). Only respond with text."#;

    /// PARTIAL_COMPACT_PROMPT - 部分对话摘要（from 方向）
    ///
    /// 场景：仅摘要从指定消息开始到当前的消息
    /// 适用：对话前半段已被压缩过
    pub const PARTIAL: &'static str = r#"Please summarize the conversation from the marked point to the present.

Follow the same format as the base compact prompt.
IMPORTANT: Do NOT use any tools. Only respond with text."#;

    /// PARTIAL_COMPACT_UP_TO_PROMPT - 部分对话摘要（up_to 方向）
    ///
    /// 场景：摘要从对话开始到指定消息的内容
    /// 适用：保留最近完整消息
    pub const PARTIAL_UP_TO: &'static str = r#"Please summarize the conversation from the beginning up to the marked point.

Follow the same format as the base compact prompt.
IMPORTANT: Do NOT use any tools. Only respond with text."#;

    /// 双阶段输出结构说明
    pub fn two_stage_rationale() -> &'static str {
        "<analysis> 块作为 Chain-of-Thought 的载体提升了摘要质量，但不会进入最终的上下文窗口，避免浪费令牌。
        这种设计模式：思考是过程，摘要是结果。过程不计费，结果才计入上下文。"
    }
}

/// 双阶段压缩输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwoStageOutput {
    /// 分析块（思维草稿本，会被丢弃）
    pub analysis: Option<String>,
    /// 摘要块（正式输出，保留）
    pub summary: String,
    /// 是否成功提取
    pub extraction_success: bool,
}

impl TwoStageOutput {
    /// 从 LLM 原始输出解析双阶段结构
    pub fn parse_from_llm_output(output: &str) -> Self {
        // 提取 <analysis> 块
        let analysis = Self::extract_tagged_block(output, "analysis");

        // 提取 <summary> 块
        let summary =
            Self::extract_tagged_block(output, "summary").unwrap_or_else(|| output.to_string());

        let extraction_success = !summary.is_empty();

        TwoStageOutput {
            analysis,
            summary,
            extraction_success,
        }
    }

    /// 提取 XML 标签块
    fn extract_tagged_block(text: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        if let Some(start) = text.find(&open_tag) {
            if let Some(end) = text.find(&close_tag) {
                if start < end {
                    let content_start = start + open_tag.len();
                    let content = text[content_start..end].trim();
                    return Some(content.to_string());
                }
            }
        }

        None
    }

    /// 获取最终输出（丢弃 analysis，只保留 summary）
    pub fn finalize(self) -> String {
        self.summary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compressible_tool_type() {
        assert!(CompressibleToolType::is_compressible("Read"));
        assert!(CompressibleToolType::is_compressible("bash"));
        assert!(CompressibleToolType::is_compressible("GLOB"));
        assert!(!CompressibleToolType::is_compressible("TodoWrite"));
    }

    #[test]
    fn test_snip_marker() {
        assert_eq!(SNIP_MARKER_TEXT, "[Old tool result content cleared]");
    }

    #[test]
    fn test_snip_execution() {
        let config = SnipConfig {
            message_uuids: vec!["msg-1".to_string(), "msg-2".to_string()],
            preserve_structure: true,
        };

        let result = SnipCompression::execute(&config);
        assert_eq!(result.cleared_count, 2);
        assert!(result.estimated_tokens_freed > 0);
    }

    #[test]
    fn test_micro_compact_evaluation() {
        let mut compression = MicroCompactCompression::new(MicroCompactConfig::default());

        // 初始状态
        let eval = compression.evaluate();
        assert!(!eval.should_trigger);

        // 设置很久以前的时间
        let old_time = Instant::now() - Duration::from_secs(7200); // 2 小时前
        compression.update_last_assistant_time(old_time);

        // 现在应该触发
        let eval = compression.evaluate();
        assert!(eval.should_trigger);
    }

    #[test]
    fn test_collapse_trigger() {
        let compression = CollapseCompression::new(CollapseConfig::default());

        // 85% 不应触发
        assert!(!compression.should_trigger(0.85));

        // 90% 应触发
        assert!(compression.should_trigger(0.90));

        // 95% 应阻止 spawn
        assert!(compression.should_block_spawn(0.95));
    }

    #[test]
    fn test_prompt_templates() {
        assert!(CompactPrompts::BASE.contains("<analysis>"));
        assert!(CompactPrompts::BASE.contains("<summary>"));
        assert!(CompactPrompts::BASE.contains("Do NOT use any tools"));
    }

    #[test]
    fn test_two_stage_output_parse() {
        let llm_output = r#"<analysis>
This is the thinking process that should be discarded.
</analysis>

<summary>
## Goals
This is the actual summary that should be kept.
</summary>"#;

        let parsed = TwoStageOutput::parse_from_llm_output(llm_output);

        assert!(parsed.analysis.is_some());
        assert!(parsed.summary.contains("## Goals"));
        assert!(parsed.extraction_success);

        let finalized = parsed.finalize();
        assert!(!finalized.contains("analysis"));
        assert!(finalized.contains("## Goals"));
    }

    #[test]
    fn test_two_stage_output_missing_analysis() {
        let llm_output = r"<summary>Only summary provided</summary>";

        let parsed = TwoStageOutput::parse_from_llm_output(llm_output);

        assert!(parsed.analysis.is_none());
        assert_eq!(parsed.summary, "Only summary provided");
    }
}
