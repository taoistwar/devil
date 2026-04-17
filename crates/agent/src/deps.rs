//! 依赖注入模块
//! 
//! 定义 QueryDeps 接口，实现依赖注入模式：
//! - 模型调用函数
//! - 轻量压缩函数 (MicroCompact)
//! - 自动压缩函数 (AutoCompact)
//! - UUID 生成器
//! 
//! 依赖注入使得测试可以注入 fake 实现，避免模块级别的 mock 样板代码

use anyhow::Result;
use async_trait::async_trait;
use crate::message::{Message, AssistantMessage};
use crate::state::{Continue, Terminal};

/// 模型调用结果
#[derive(Debug, Clone)]
pub struct ModelCallResult {
    /// 助手消息
    pub assistant_message: AssistantMessage,
    /// 使用的输入 token 数
    pub input_tokens: usize,
    /// 使用的输出 token 数
    pub output_tokens: usize,
    /// 停止原因
    pub stop_reason: Option<String>,
}

/// 模型调用参数
#[derive(Debug, Clone)]
pub struct ModelCallParams {
    /// 系统提示词
    pub system_prompt: String,
    /// 消息历史
    pub messages: Vec<Message>,
    /// 最大输出 token 数
    pub max_tokens: usize,
    /// 模型名称
    pub model: String,
}

/// 压缩结果
#[derive(Debug, Clone)]
pub struct CompactResult {
    /// 压缩后的消息列表
    pub messages: Vec<Message>,
    /// 是否成功压缩
    pub success: bool,
    /// 压缩前后的 token 数变化
    pub token_reduction: Option<usize>,
}

/// UUID 生成 Trait
pub trait UuidGenerator: Send + Sync {
    /// 生成新的 UUID
    fn generate(&self) -> String;
}

/// 生产环境的 UUID 生成器
#[derive(Default)]
pub struct ProductionUuidGenerator;

impl UuidGenerator for ProductionUuidGenerator {
    fn generate(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}

/// 依赖注入接口
/// 
/// 包含对话循环所需的四个核心依赖：
/// 1. 模型调用函数
/// 2. MicroCompact 压缩函数
/// 3. AutoCompact 压缩函数
/// 4. UUID 生成器
#[async_trait]
pub trait QueryDeps: Send + Sync {
    /// 调用模型 API
    async fn call_model(
        &self,
        params: ModelCallParams,
    ) -> Result<ModelCallResult>;

    /// 执行 MicroCompact（轻量级压缩）
    /// 
    /// MicroCompact 利用缓存编辑技术减少 token 消耗
    /// 尽量复用 API 侧已缓存的 token，避免缓存全面失效
    async fn micro_compact(&self, messages: Vec<Message>) -> Result<CompactResult>;

    /// 执行 AutoCompact（全量压缩）
    /// 
    /// 当上下文超过阈值时，将历史对话摘要为压缩后的消息
    async fn auto_compact(&self, messages: Vec<Message>) -> Result<CompactResult>;

    /// 生成 UUID
    fn generate_uuid(&self) -> String;
}

/// 生产环境的依赖实现
pub struct ProductionDeps {
    /// UUID 生成器
    pub uuid_generator: Box<dyn UuidGenerator>,
}

impl ProductionDeps {
    /// 创建生产环境依赖
    pub fn new() -> Self {
        Self {
            uuid_generator: Box::new(ProductionUuidGenerator::default()),
        }
    }
}

impl Default for ProductionDeps {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl QueryDeps for ProductionDeps {
    async fn call_model(
        &self,
        _params: ModelCallParams,
    ) -> Result<ModelCallResult> {
        // TODO: 实现真实的 API 调用
        // 这里应该调用 Anthropic API
        anyhow::bail!("Model call not implemented")
    }

    async fn micro_compact(
        &self,
        messages: Vec<Message>,
    ) -> Result<CompactResult> {
        // TODO: 实现 MicroCompact 压缩逻辑
        Ok(CompactResult {
            messages,
            success: false,
            token_reduction: None,
        })
    }

    async fn auto_compact(
        &self,
        messages: Vec<Message>,
    ) -> Result<CompactResult> {
        // TODO: 实现 AutoCompact 压缩逻辑
        Ok(CompactResult {
            messages,
            success: false,
            token_reduction: None,
        })
    }

    fn generate_uuid(&self) -> String {
        self.uuid_generator.generate()
    }
}

/// 测试用的依赖实现
/// 
/// 允许测试代码注入自定义行为，无需 mock 模块级别的函数
pub struct TestDeps {
    /// 模型调用函数
    pub call_model_fn: Box<
        dyn Fn(ModelCallParams) -> Result<ModelCallResult> + Send + Sync,
    >,
    /// MicroCompact 函数
    pub micro_compact_fn: Box<
        dyn Fn(Vec<Message>) -> Result<CompactResult> + Send + Sync,
    >,
    /// AutoCompact 函数
    pub auto_compact_fn: Box<
        dyn Fn(Vec<Message>) -> Result<CompactResult> + Send + Sync,
    >,
    /// UUID 生成函数
    pub generate_uuid_fn: Box<dyn Fn() -> String + Send + Sync>,
}

impl TestDeps {
    /// 创建测试依赖
    pub fn new(
        call_model_fn: impl Fn(ModelCallParams) -> Result<ModelCallResult>
            + Send
            + Sync
            + 'static,
        micro_compact_fn: impl Fn(Vec<Message>) -> Result<CompactResult>
            + Send
            + Sync
            + 'static,
        auto_compact_fn: impl Fn(Vec<Message>) -> Result<CompactResult>
            + Send
            + Sync
            + 'static,
        generate_uuid_fn: impl Fn() -> String + Send + Sync + 'static,
    ) -> Self {
        Self {
            call_model_fn: Box::new(call_model_fn),
            micro_compact_fn: Box::new(micro_compact_fn),
            auto_compact_fn: Box::new(auto_compact_fn),
            generate_uuid_fn: Box::new(generate_uuid_fn),
        }
    }

    /// 创建只返回空实现的测试依赖
    pub fn empty() -> Self {
        Self::new(
            |_| Ok(ModelCallResult {
                assistant_message: AssistantMessage::text(""),
                input_tokens: 0,
                output_tokens: 0,
                stop_reason: Some("stop_sequence".to_string()),
            }),
            |msgs| Ok(CompactResult {
                messages: msgs,
                success: false,
                token_reduction: None,
            }),
            |msgs| Ok(CompactResult {
                messages: msgs,
                success: false,
                token_reduction: None,
            }),
            || "test-uuid".to_string(),
        )
    }
}

#[async_trait]
impl QueryDeps for TestDeps {
    async fn call_model(&self, params: ModelCallParams) -> Result<ModelCallResult> {
        (self.call_model_fn)(params)
    }

    async fn micro_compact(&self, messages: Vec<Message>) -> Result<CompactResult> {
        (self.micro_compact_fn)(messages)
    }

    async fn auto_compact(&self, messages: Vec<Message>) -> Result<CompactResult> {
        (self.auto_compact_fn)(messages)
    }

    fn generate_uuid(&self) -> String {
        (self.generate_uuid_fn)()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::UserMessage;

    #[test]
    fn test_production_deps_creation() {
        let deps = ProductionDeps::new();
        let uuid = deps.generate_uuid();
        assert!(!uuid.is_empty());
        assert_ne!(uuid, deps.generate_uuid()); // UUID 应该唯一
    }

    #[tokio::test]
    async fn test_test_deps() {
        let test_deps = TestDeps::new(
            |params| {
                Ok(ModelCallResult {
                    assistant_message: AssistantMessage::text(format!(
                        "Called with {} messages",
                        params.messages.len()
                    )),
                    input_tokens: 100,
                    output_tokens: 50,
                    stop_reason: Some("stop_sequence".to_string()),
                })
            },
            |msgs| {
                Ok(CompactResult {
                    messages: msgs,
                    success: true,
                    token_reduction: Some(1000),
                })
            },
            |msgs| Ok(CompactResult {
                messages: msgs,
                success: false,
                token_reduction: None,
            }),
            || "fixed-uuid".to_string(),
        );

        let messages = vec![Message::User(UserMessage::text("test"))];
        let result = test_deps
            .call_model(ModelCallParams {
                system_prompt: "test".to_string(),
                messages: messages.clone(),
                max_tokens: 1000,
                model: "test-model".to_string(),
            })
            .await
            .unwrap();

        assert!(result.assistant_message.text_content().contains("Called with"));
        assert_eq!(result.input_tokens, 100);
    }
}
