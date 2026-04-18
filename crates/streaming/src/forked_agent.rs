//! ForkedAgent - 子任务执行器
//!
//! 实现：
//! - Fork 模式字节级一致性
//! - Cache-safe params 共享
//! - contentReplacementState 克隆

use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cost_tracking::TokenUsage;
use crate::query_engine::Message;

/// Fork 上下文
pub struct ForkContext {
    /// 消息前缀（用于缓存命中）
    pub messages: Vec<Message>,
    /// Cache-safe 参数
    pub cache_safe_params: CacheSafeParams,
    /// 内容替换状态（克隆以保证字节级一致）
    pub content_replacement_state: Arc<ContentReplacementState>,
}

/// Cache-safe 参数（缓存键五维度）
pub struct CacheSafeParams {
    /// System prompt
    pub system_prompt: String,
    /// User context
    pub user_context: String,
    /// System context
    pub system_context: String,
    /// Tool use context
    pub tool_use_context: String,
    /// Fork context messages
    pub fork_context_messages: Vec<Message>,
}

impl CacheSafeParams {
    /// 创建缓存键
    pub fn cache_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        self.system_prompt.hash(&mut hasher);
        self.user_context.hash(&mut hasher);
        self.system_context.hash(&mut hasher);
        self.tool_use_context.hash(&mut hasher);

        // Messages 的数量也加入哈希
        self.fork_context_messages.len().hash(&mut hasher);

        format!("{:016x}", hasher.finish())
    }
}

/// 内容替换状态
#[derive(Debug, Clone, Default)]
pub struct ContentReplacementState {
    /// 工具使用 ID 替换映射
    pub tool_use_replacements: std::collections::HashMap<String, String>,
    /// 已处理的消息 ID
    pub processed_message_ids: std::collections::HashSet<String>,
}

/// ForkedAgent 配置
pub struct ForkedAgentConfig {
    /// 模型名称
    pub model: String,
    /// 最大输出 token
    pub max_tokens: u32,
    /// Thinking 配置
    pub thinking_config: Option<ThinkingConfig>,
    /// 跳过缓存写入
    pub skip_cache_write: bool,
}

/// Thinking 配置
#[derive(Debug, Clone)]
pub struct ThinkingConfig {
    /// 启用 thinking
    pub enabled: bool,
    /// Token 预算
    pub budget_tokens: Option<u32>,
}

/// ForkedAgent 选项
pub struct AgentOptions {
    /// 最大输出 token
    pub max_output_tokens: Option<u32>,
    /// Thinking 配置
    pub thinking_config: Option<ThinkingConfig>,
}

/// ForkedAgent 结果
pub struct ForkedAgentResult {
    /// 响应消息
    pub message: String,
    /// Token 用量
    pub usage: TokenUsage,
    /// 缓存命中率
    pub cache_hit_rate: f64,
    /// 工具调用
    pub tool_calls: Vec<ToolCall>,
}

/// 工具调用
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// ForkedAgent - 子任务执行器
pub struct ForkedAgent {
    /// Fork 上下文
    context: ForkContext,
    /// 配置
    config: ForkedAgentConfig,
}

impl ForkedAgent {
    /// 创建新的 ForkedAgent
    pub fn new(context: ForkContext, config: ForkedAgentConfig) -> Self {
        Self { context, config }
    }

    /// 运行 ForkedAgent
    pub async fn run(&self, prompt: String) -> Result<ForkedAgentResult> {
        info!("Running forked agent");

        // 构建完整的消息列表
        // fork_context_messages 与 prompt 拼接
        let mut messages = self.context.cache_safe_params.fork_context_messages.clone();

        // 添加当前 prompt
        messages.push(Message::User {
            content: prompt.clone(),
        });

        debug!(
            "Fork context: {} messages, cache_key={}",
            messages.len(),
            self.context.cache_safe_params.cache_key()
        );

        // 克隆 content_replacement_state 以保证字节级一致性
        // 这对于缓存命中至关重要
        let replacement_state = self.context.content_replacement_state.clone();

        // TODO: 实际调用模型 API
        // 这里返回模拟结果
        Ok(ForkedAgentResult {
            message: format!("Fork response to: {}", prompt),
            usage: TokenUsage::new(),
            cache_hit_rate: 0.0,
            tool_calls: Vec::new(),
        })
    }

    /// 获取缓存命中率
    pub fn calculate_cache_hit_rate(cache_read: u32, input: u32, cache_creation: u32) -> f64 {
        let total = cache_read + input + cache_creation;
        if total == 0 {
            0.0
        } else {
            cache_read as f64 / total as f64
        }
    }
}

/// 运行 Forked Agent 的便捷函数
pub async fn run_forked_agent(
    context: ForkContext,
    prompt: String,
    options: AgentOptions,
) -> Result<ForkedAgentResult> {
    let config = ForkedAgentConfig {
        model: "claude-3-sonnet".to_string(),
        max_tokens: options.max_output_tokens.unwrap_or(4096),
        thinking_config: options.thinking_config,
        skip_cache_write: false,
    };

    let agent = ForkedAgent::new(context, config);
    agent.run(prompt).await
}

/// 构建 cache-safe params
pub fn build_cache_safe_params(
    system_prompt: String,
    user_context: String,
    system_context: String,
    tool_use_context: String,
    fork_context_messages: Vec<Message>,
) -> CacheSafeParams {
    CacheSafeParams {
        system_prompt,
        user_context,
        system_context,
        tool_use_context,
        fork_context_messages,
    }
}

/// 克隆 content_replacement_state 以保证字节级一致性
pub fn clone_replacement_state(state: &ContentReplacementState) -> ContentReplacementState {
    ContentReplacementState {
        tool_use_replacements: state.tool_use_replacements.clone(),
        processed_message_ids: state.processed_message_ids.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_generation() {
        let params = CacheSafeParams {
            system_prompt: "You are an assistant".to_string(),
            user_context: "user123".to_string(),
            system_context: "default".to_string(),
            tool_use_context: "read,grep".to_string(),
            fork_context_messages: vec![],
        };

        let key1 = params.cache_key();
        let key2 = params.cache_key();

        // 相同参数应该生成相同的键
        assert_eq!(key1, key2);

        // 修改参数应该生成不同的键
        let mut params2 = params.clone();
        params2.system_prompt = "Different prompt".to_string();
        let key3 = params2.cache_key();

        assert_ne!(key1, key3);
    }

    #[test]
    fn test_clone_replacement_state() {
        let mut state = ContentReplacementState::default();
        state
            .tool_use_replacements
            .insert("id1".to_string(), "val1".to_string());
        state.processed_message_ids.insert("msg1".to_string());

        let cloned = clone_replacement_state(&state);

        assert!(cloned.tool_use_replacements.contains_key("id1"));
        assert!(cloned.processed_message_ids.contains("msg1"));
    }

    #[test]
    fn test_cache_hit_rate_calculation() {
        // 高命中率
        let rate = ForkedAgent::calculate_cache_hit_rate(800, 100, 100);
        assert!((rate - 0.8).abs() < 0.001);

        // 低命中率
        let rate = ForkedAgent::calculate_cache_hit_rate(100, 800, 100);
        assert!((rate - 0.1).abs() < 0.001);

        // 零值处理
        let rate = ForkedAgent::calculate_cache_hit_rate(0, 0, 0);
        assert_eq!(rate, 0.0);
    }

    #[tokio::test]
    async fn test_forked_agent_creation() {
        let context = ForkContext {
            messages: vec![],
            cache_safe_params: CacheSafeParams {
                system_prompt: "test".to_string(),
                user_context: "user".to_string(),
                system_context: "ctx".to_string(),
                tool_use_context: "tools".to_string(),
                fork_context_messages: vec![],
            },
            content_replacement_state: Arc::new(ContentReplacementState::default()),
        };

        let config = ForkedAgentConfig {
            model: "claude-3-sonnet".to_string(),
            max_tokens: 1024,
            thinking_config: None,
            skip_cache_write: false,
        };

        let agent = ForkedAgent::new(context, config);

        let result = agent.run("test prompt".to_string()).await.unwrap();

        assert!(result.message.contains("test prompt"));
    }
}
