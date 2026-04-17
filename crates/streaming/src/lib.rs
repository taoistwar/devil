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

pub mod query_engine;
pub mod streaming_tool_executor;
pub mod forked_agent;
pub mod prefetch;
pub mod cost_tracking;
pub mod cache_optimizer;
pub mod mcp_integration;
pub mod anthropic_deps;

pub use query_engine::{QueryEngine, QueryDeps, StreamEvent};
pub use streaming_tool_executor::{StreamingToolExecutor, TrackedTool, ToolState};
pub use forked_agent::{ForkedAgent, ForkContext, ForkedAgentResult, CacheSafeParams};
pub use prefetch::ParallelPrefetcher;
pub use cost_tracking::{TokenUsage, UsageDelta, CostTracker};
pub use cache_optimizer::{CacheOptimizer, CacheMetrics};
pub use mcp_integration::{
    McpQueryDeps,
    McpToolConverter,
    McpStreamingIntegration,
    create_mcp_query_engine,
};
pub use anthropic_deps::AnthropicQueryDeps;

/// 流式处理版本
pub const STREAMING_VERSION: &str = env!("CARGO_PKG_VERSION");
