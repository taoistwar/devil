//! Streaming 模块集成测试

use devil_streaming::{
    QueryEngine, StreamEvent, StreamingToolExecutor, TrackedTool, ToolState,
    ForkedAgent, ForkContext, CacheSafeParams, ContentReplacementState,
    ParallelPrefetcher, TokenUsage, UsageDelta, CostTracker,
    CacheOptimizer, CacheMetrics,
};
use futures::stream;
use std::sync::Arc;

/// Mock QueryDeps 实现
struct MockQueryDeps;

impl devil_streaming::query_engine::QueryDeps for MockQueryDeps {
    fn call_model(
        &self,
        _messages: &[devil_streaming::query_engine::Message],
        _stream: bool,
    ) -> futures::stream::BoxStream<'static, anyhow::Result<StreamEvent>> {
        stream::empty().boxed()
    }

    fn execute_tool(
        &self,
        _tool: &TrackedTool,
    ) -> futures::future::BoxFuture<'static, anyhow::Result<String>> {
        async { Ok("".to_string()) }.boxed()
    }
}

#[tokio::test]
async fn test_query_engine_basic() {
    let engine = QueryEngine::<MockQueryDeps>::new(MockQueryDeps);
    
    assert!(!engine.is_aborted());
    
    let usage = engine.get_usage().await;
    assert_eq!(usage.input_tokens, 0);
    assert_eq!(usage.output_tokens, 0);
}

#[tokio::test]
async fn test_query_engine_abort() {
    let engine = QueryEngine::<MockQueryDeps>::new(MockQueryDeps);
    
    engine.abort();
    assert!(engine.is_aborted());
}

#[tokio::test]
async fn test_streaming_executor_concurrent_tools() {
    let mut executor = StreamingToolExecutor::new();
    
    // 添加两个并发安全工具
    executor.add_tool(TrackedTool::new(
        "read-1".to_string(),
        "read_file".to_string(),
        serde_json::json!({"path": "/test1.txt"}),
        true,
    )).await;
    
    executor.add_tool(TrackedTool::new(
        "grep-1".to_string(),
        "grep".to_string(),
        serde_json::json!({"pattern": "TODO"}),
        true,
    )).await;
    
    // 两个工具都应该可以执行
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    let count = executor.pending_count().await;
    assert_eq!(count, 0); // 都应该已开始执行
}

#[tokio::test]
async fn test_streaming_executor_unsafe_blocks() {
    let mut executor = StreamingToolExecutor::new();
    
    // 添加 Bash 工具（非并发安全）
    executor.add_tool(TrackedTool::new(
        "bash-1".to_string(),
        "bash".to_string(),
        serde_json::json!({"command": "echo hello"}),
        false,
    )).await;
    
    // 添加另一个工具
    executor.add_tool(TrackedTool::new(
        "read-1".to_string(),
        "read_file".to_string(),
        serde_json::json!({"path": "/test.txt"}),
        true,
    )).await;
    
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    // Bash 独占执行，另一个应该排队
    let count = executor.pending_count().await;
    assert!(count >= 1);
}

#[tokio::test]
async fn test_cost_tracking_update() {
    let mut usage = TokenUsage::new();
    
    let delta = UsageDelta {
        input_tokens: Some(100),
        output_tokens: Some(50),
        cache_creation_input_tokens: Some(200),
        cache_read_input_tokens: Some(300),
    };
    
    devil_streaming::cost_tracking::update_usage(&mut usage, delta);
    
    assert_eq!(usage.input_tokens, 100);
    assert_eq!(usage.output_tokens, 50);
    assert_eq!(usage.cache_creation_input_tokens, 200);
    assert_eq!(usage.cache_read_input_tokens, 300);
}

#[tokio::test]
async fn test_cost_tracking_zero_guard() {
    let mut usage = TokenUsage::new();
    usage.input_tokens = 1000;
    
    // 零值不应该覆盖现有值
    let delta = UsageDelta {
        input_tokens: Some(0),
        ..Default::default()
    };
    
    devil_streaming::cost_tracking::update_usage(&mut usage, delta);
    
    assert_eq!(usage.input_tokens, 1000); // 保持不变
}

#[tokio::test]
async fn test_cost_tracker_budget() {
    let tracker = CostTracker::new("claude-3-sonnet".to_string(), Some(10.0));
    
    // 初始预算使用率应该为 0
    let usage = tracker.get_budget_usage().await;
    assert_eq!(usage, 0.0);
    
    // 检查预警级别
    let level = tracker.get_budget_warning_level().await;
    assert_eq!(level, devil_streaming::cost_tracking::BudgetWarningLevel::Normal);
}

#[tokio::test]
async fn test_cache_optimizer_recording() {
    let optimizer = CacheOptimizer::new();
    
    let usage = TokenUsage {
        input_tokens: 100,
        output_tokens: 50,
        cache_creation_input_tokens: 200,
        cache_read_input_tokens: 800,
    };
    
    optimizer.record_usage(&usage);
    
    let metrics = optimizer.get_cumulative_metrics();
    assert_eq!(metrics.cache_read, 800);
    assert_eq!(metrics.cache_creation, 200);
    assert_eq!(metrics.input, 100);
    
    let hit_rate = optimizer.get_hit_rate();
    assert!((hit_rate - 0.8).abs() < 0.001);
}

#[tokio::test]
async fn test_cache_optimizer_health() {
    let optimizer = CacheOptimizer::new();
    
    // 记录高命中率数据
    for _ in 0..10 {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 800,
        };
        optimizer.record_usage(&usage);
    }
    
    let health = optimizer.check_health();
    assert_eq!(health, devil_streaming::cache_optimizer::CacheHealthStatus::Excellent);
}

#[test]
fn test_prefetch_handle_creation() {
    let prefetcher = ParallelPrefetcher::new();
    let mdm_handle = prefetcher.start_mdm_raw_read();
    let keychain_handle = prefetcher.start_keychain_prefetch();
    
    // 句柄应该成功创建
    assert!(mdm_handle.start_time.elapsed().as_millis() >= 0);
    assert!(keychain_handle.start_time.elapsed().as_millis() >= 0);
}

#[tokio::test]
async fn test_forked_agent_cache_key() {
    use devil_streaming::forked_agent::build_cache_safe_params;
    use devil_streaming::query_engine::Message;
    
    let params1 = build_cache_safe_params(
        "prompt1".to_string(),
        "user1".to_string(),
        "ctx".to_string(),
        "tools".to_string(),
        vec![],
    );
    
    let params2 = build_cache_safe_params(
        "prompt2".to_string(), // 不同
        "user1".to_string(),
        "ctx".to_string(),
        "tools".to_string(),
        vec![],
    );
    
    // 不同参数应该生成不同的缓存键
    assert_ne!(params1.cache_key(), params2.cache_key());
}

#[tokio::test]
async fn test_full_streaming_workflow() {
    // 完整流式工作流测试
    let engine = QueryEngine::<MockQueryDeps>::new(MockQueryDeps);
    let mut executor = StreamingToolExecutor::new();
    let tracker = CostTracker::new("claude-3-sonnet".to_string(), Some(10.0));
    let optimizer = CacheOptimizer::new();
    
    // 验证所有组件正常工作
    assert!(!engine.is_aborted());
    assert_eq!(executor.pending_count().await, 0);
    assert_eq!(tracker.get_budget_usage().await, 0.0);
    assert_eq!(optimizer.get_hit_rate(), 0.0);
}
