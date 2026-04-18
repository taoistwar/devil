//! QueryEngine - 查询生命周期管理者
//!
//! 负责：
//! - 管理对话历史消息
//! - 流式生成响应（AsyncGenerator）
//! - 维护会话状态（usage、fileCache、skills 等）
//! - 中止控制

use anyhow::{Context, Result};
use futures::stream::Stream;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::cost_tracking::{TokenUsage, UsageDelta};
use crate::streaming_tool_executor::TrackedTool;

/// 消息类型
#[derive(Debug, Clone)]
pub enum Message {
    User {
        content: String,
    },
    Assistant {
        content: Vec<ContentBlock>,
        usage: Option<TokenUsage>,
    },
    ToolResult {
        tool_use_id: String,
        content: String,
        is_error: bool,
    },
}

/// 内容块类型
#[derive(Debug, Clone)]
pub enum ContentBlock {
    Text {
        text: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

/// 流式事件类型
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// 消息开始
    MessageStart { id: String },
    /// 内容块增量
    ContentBlockDelta {
        block_type: BlockType,
        delta: ContentDelta,
    },
    /// 消息增量（usage 更新）
    MessageDelta {
        usage: UsageDelta,
        stop_reason: Option<StopReason>,
    },
    /// 消息结束
    MessageStop,
    /// 工具调用
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    /// Progress 消息
    Progress { message: String },
}

/// 块类型
#[derive(Debug, Clone)]
pub enum BlockType {
    Text,
    ToolUse,
}

/// 内容增量
#[derive(Debug, Clone)]
pub enum ContentDelta {
    TextDelta { text: String },
    InputDelta { partial_json: String },
}

/// 终止原因
#[derive(Debug, Clone)]
pub enum StopReason {
    EndTurn,
    ToolUse,
    MaxTokens,
    StopSequence,
}

/// 文件状态
#[derive(Debug, Clone)]
pub struct FileState {
    pub path: String,
    pub last_modified: u64,
    pub size: u64,
}

/// 依赖注入接口
pub trait QueryDeps: Send {
    /// 调用模型 API
    fn call_model(
        &self,
        messages: &[Message],
        stream: bool,
    ) -> futures::stream::BoxStream<'static, Result<StreamEvent>>;

    /// 执行工具
    fn execute_tool(
        &self,
        tool: &TrackedTool,
    ) -> futures::future::BoxFuture<'static, Result<String>>;
}

/// 中止控制器
pub struct AbortController {
    aborted: AtomicBool,
}

impl AbortController {
    pub fn new() -> Self {
        Self {
            aborted: AtomicBool::new(false),
        }
    }

    pub fn abort(&self) {
        self.aborted.store(true, Ordering::Relaxed);
        info!("Query aborted");
    }

    pub fn is_aborted(&self) -> bool {
        self.aborted.load(Ordering::Relaxed)
    }
}

impl Default for AbortController {
    fn default() -> Self {
        Self::new()
    }
}

/// QueryEngine - 查询生命周期管理者
pub struct QueryEngine<D: QueryDeps> {
    /// 对话历史
    messages: Arc<RwLock<Vec<Message>>>,
    /// 中止控制器
    abort_controller: Arc<AbortController>,
    /// 权限拒绝记录
    denied_permissions: Arc<RwLock<HashSet<String>>>,
    /// 用量统计
    usage: Arc<RwLock<TokenUsage>>,
    /// 文件状态缓存
    file_state_cache: Arc<RwLock<HashMap<String, FileState>>>,
    /// 已发现的技能
    discovered_skills: Arc<RwLock<HashSet<String>>>,
    /// 依赖注入
    deps: D,
    /// 当前消息 ID
    current_message_id: Arc<Mutex<Option<String>>>,
}

impl<D: QueryDeps> QueryEngine<D> {
    /// 创建新的 QueryEngine
    pub fn new(deps: D) -> Self {
        Self {
            messages: Arc::new(RwLock::new(Vec::new())),
            abort_controller: Arc::new(AbortController::new()),
            denied_permissions: Arc::new(RwLock::new(HashSet::new())),
            usage: Arc::new(RwLock::new(TokenUsage::default())),
            file_state_cache: Arc::new(RwLock::new(HashMap::new())),
            discovered_skills: Arc::new(RwLock::new(HashSet::new())),
            deps,
            current_message_id: Arc::new(Mutex::new(None)),
        }
    }

    /// 提交消息（流式生成器）
    pub async fn submit_message(
        &mut self,
        message: String,
    ) -> Result<impl Stream<Item = StreamEvent>> {
        if self.abort_controller.is_aborted() {
            anyhow::bail!("Query was previously aborted");
        }

        // 添加用户消息
        let user_msg = Message::User {
            content: message.clone(),
        };
        self.messages.write().await.push(user_msg);

        info!("Submitting message to QueryEngine");

        // 调用模型 API（流式）
        let messages = self.messages.read().await.clone();
        let stream = self.deps.call_model(&messages, true);

        // 包装为处理流
        Ok(self.process_stream(stream))
    }

    /// 处理流式响应
    fn process_stream(
        &self,
        stream: futures::stream::BoxStream<'static, Result<StreamEvent>>,
    ) -> impl Stream<Item = StreamEvent> {
        use futures::stream::StreamExt;

        let usage = self.usage.clone();
        let messages = self.messages.clone();
        let abort = self.abort_controller.clone();

        stream.filter_map(move |event| {
            let usage = usage.clone();
            let messages = messages.clone();
            let abort = abort.clone();

            async move {
                if abort.is_aborted() {
                    return None;
                }

                match event {
                    Ok(e) => {
                        // 处理 usage 更新
                        if let StreamEvent::MessageDelta { usage: delta, .. } = &e {
                            let mut current = usage.write().await;
                            crate::cost_tracking::update_usage(&mut *current, delta.clone());
                        }

                        Some(e)
                    }
                    Err(e) => {
                        error!("Stream error: {}", e);
                        None
                    }
                }
            }
        })
    }

    /// 中止当前查询
    pub fn abort(&self) {
        self.abort_controller.abort();
    }

    /// 获取用量统计
    pub async fn get_usage(&self) -> TokenUsage {
        self.usage.read().await.clone()
    }

    /// 添加消息
    pub async fn add_message(&self, message: Message) {
        self.messages.write().await.push(message);
    }

    /// 获取消息历史
    pub async fn get_messages(&self) -> Vec<Message> {
        self.messages.read().await.clone()
    }

    /// 记录拒绝的权限
    pub async fn record_denied_permission(&self, permission: String) {
        self.denied_permissions.write().await.insert(permission);
    }

    /// 检查权限是否被拒绝
    pub async fn is_permission_denied(&self, permission: &str) -> bool {
        self.denied_permissions.read().await.contains(permission)
    }

    /// 更新文件缓存
    pub async fn update_file_cache(&self, path: String, state: FileState) {
        self.file_state_cache.write().await.insert(path, state);
    }

    /// 获取文件缓存
    pub async fn get_file_state(&self, path: &str) -> Option<FileState> {
        self.file_state_cache.read().await.get(path).cloned()
    }

    /// 标记技能已发现
    pub async fn mark_skill_discovered(&self, skill_name: String) {
        self.discovered_skills.write().await.insert(skill_name);
    }

    /// 检查技能是否已发现
    pub async fn is_skill_discovered(&self, skill_name: &str) -> bool {
        self.discovered_skills.read().await.contains(skill_name)
    }

    /// 检查是否已中止
    pub fn is_aborted(&self) -> bool {
        self.abort_controller.is_aborted()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream;

    struct MockDeps;

    impl QueryDeps for MockDeps {
        fn call_model(
            &self,
            _messages: &[Message],
            _stream: bool,
        ) -> futures::stream::BoxStream<'static, Result<StreamEvent>> {
            stream::empty().boxed()
        }

        fn execute_tool(
            &self,
            _tool: &TrackedTool,
        ) -> futures::future::BoxFuture<'static, Result<String>> {
            async { Ok("".to_string()) }.boxed()
        }
    }

    #[tokio::test]
    async fn test_query_engine_creation() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);
        assert!(!engine.is_aborted());

        let usage = engine.get_usage().await;
        assert_eq!(usage.input_tokens, 0);
    }

    #[tokio::test]
    async fn test_abort_controller() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);
        assert!(!engine.is_aborted());

        engine.abort();
        assert!(engine.is_aborted());
    }

    #[tokio::test]
    async fn test_message_history() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);

        let msg = Message::User {
            content: "test".to_string(),
        };
        engine.add_message(msg).await;

        let messages = engine.get_messages().await;
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            Message::User { content } => assert_eq!(content, "test"),
            _ => panic!("Expected User message"),
        }
    }

    #[tokio::test]
    async fn test_permission_tracking() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);

        assert!(!engine.is_permission_denied("bash").await);

        engine.record_denied_permission("bash".to_string()).await;

        assert!(engine.is_permission_denied("bash").await);
    }

    #[tokio::test]
    async fn test_file_cache() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);

        let state = FileState {
            path: "/test.txt".to_string(),
            last_modified: 12345,
            size: 1024,
        };

        engine
            .update_file_cache("/test.txt".to_string(), state.clone())
            .await;

        let cached = engine.get_file_state("/test.txt").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().size, 1024);
    }

    #[tokio::test]
    async fn test_skill_discovery() {
        let engine = QueryEngine::<MockDeps>::new(MockDeps);

        assert!(!engine.is_skill_discovered("debug").await);

        engine.mark_skill_discovered("debug".to_string()).await;

        assert!(engine.is_skill_discovered("debug").await);
    }
}
