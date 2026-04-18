//! 流式处理架构与性能优化模块
//!
//! 实现第 13 章核心功能：
//! - QueryEngine: 查询生命周期管理者
//! - StreamingToolExecutor: 工具并发执行器
//! - ForkedAgent: 子任务执行器
//! - ParallelPrefetcher: 并行预取器
//! - CostTracker: 成本追踪
//! - CacheOptimizer: 缓存优化
//! - McpIntegration: MCP 集成模块（第 12 章）
//! - AnthropicDeps: Anthropic API 集成

pub mod anthropic_deps;
pub mod cache_optimizer;
pub mod cost_tracking;
pub mod forked_agent;
pub mod mcp_integration;
pub mod prefetch;
pub mod query_engine;
pub mod streaming_tool_executor;

pub use anthropic_deps::AnthropicQueryDeps;
pub use cache_optimizer::{CacheMetrics, CacheOptimizer};
pub use cost_tracking::{CostTracker, TokenUsage, UsageDelta};
pub use forked_agent::{CacheSafeParams, ForkContext, ForkedAgent, ForkedAgentResult};
pub use mcp_integration::{
    create_mcp_query_engine, McpQueryDeps, McpStreamingIntegration, McpToolConverter,
};
pub use prefetch::ParallelPrefetcher;
pub use query_engine::{QueryDeps, QueryEngine, StreamEvent};
pub use streaming_tool_executor::{StreamingToolExecutor, ToolState, TrackedTool};

/// 流式处理版本
pub const STREAMING_VERSION: &str = env!("CARGO_PKG_VERSION");
