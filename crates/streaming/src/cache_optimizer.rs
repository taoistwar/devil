//! 缓存优化器
//!
//! 实现：
//! - 提示缓存共享的三个维度
//! - 缓存命中率监控
//! - 缓存断裂检测

use anyhow::Result;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::cost_tracking::TokenUsage;
use crate::forked_agent::CacheSafeParams;

/// 缓存指标
pub struct CacheMetrics {
    /// 缓存读取 token
    pub cache_read: u32,
    /// 缓存创建 token
    pub cache_creation: u32,
    /// 普通输入 token
    pub input: u32,
}

impl CacheMetrics {
    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.cache_read + self.input + self.cache_creation;
        if total == 0 {
            0.0
        } else {
            self.cache_read as f64 / total as f64
        }
    }

    /// 从 TokenUsage 创建
    pub fn from_usage(usage: &TokenUsage) -> Self {
        Self {
            cache_read: usage.cache_read_input_tokens,
            cache_creation: usage.cache_creation_input_tokens,
            input: usage.input_tokens,
        }
    }

    /// 判断缓存健康状态
    pub fn health_status(&self) -> CacheHealthStatus {
        let hit_rate = self.hit_rate();

        if hit_rate >= 0.85 {
            CacheHealthStatus::Excellent
        } else if hit_rate >= 0.70 {
            CacheHealthStatus::Good
        } else if hit_rate >= 0.60 {
            CacheHealthStatus::Normal
        } else {
            CacheHealthStatus::Poor
        }
    }
}

/// 缓存健康状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheHealthStatus {
    /// > 85% - 优秀
    Excellent,
    /// 70%-85% - 良好
    Good,
    /// 60%-70% - 正常
    Normal,
    /// < 60% - 差（需要排查）
    Poor,
}

/// 缓存优化器
pub struct CacheOptimizer {
    /// 当前 cache-safe params
    current_params: Arc<RwLock<Option<CacheSafeParams>>>,
    /// 累计缓存读取 token
    total_cache_read: Arc<AtomicU64>,
    /// 累计缓存创建 token
    total_cache_creation: Arc<AtomicU64>,
    /// 累计普通输入 token
    total_input: Arc<AtomicU64>,
    /// 缓存断裂次数
    cache_breaks: Arc<AtomicU32>,
    /// 请求次数
    request_count: Arc<AtomicU64>,
}

impl CacheOptimizer {
    /// 创建新的缓存优化器
    pub fn new() -> Self {
        Self {
            current_params: Arc::new(RwLock::new(None)),
            total_cache_read: Arc::new(AtomicU64::new(0)),
            total_cache_creation: Arc::new(AtomicU64::new(0)),
            total_input: Arc::new(AtomicU64::new(0)),
            cache_breaks: Arc::new(AtomicU32::new(0)),
            request_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 更新当前 cache-safe params
    pub async fn update_params(&self, params: CacheSafeParams) {
        let mut current = self.current_params.write().await;

        // 检查是否发生变化（可能导致缓存断裂）
        if let Some(ref old) = *current {
            let old_key = old.cache_key();
            let new_key = params.cache_key();

            if old_key != new_key {
                warn!("Cache params changed: {} -> {}", old_key, new_key);
                self.cache_breaks.fetch_add(1, Ordering::Relaxed);
            }
        }

        *current = Some(params);
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 记录 token 用量
    pub fn record_usage(&self, usage: &TokenUsage) {
        self.total_cache_read
            .fetch_add(usage.cache_read_input_tokens as u64, Ordering::Relaxed);
        self.total_cache_creation
            .fetch_add(usage.cache_creation_input_tokens as u64, Ordering::Relaxed);
        self.total_input
            .fetch_add(usage.input_tokens as u64, Ordering::Relaxed);

        debug!(
            "Recorded usage: cache_read={} cache_create={} input={}",
            usage.cache_read_input_tokens, usage.cache_creation_input_tokens, usage.input_tokens
        );
    }

    /// 获取累计缓存指标
    pub fn get_cumulative_metrics(&self) -> CacheMetrics {
        CacheMetrics {
            cache_read: self.total_cache_read.load(Ordering::Relaxed) as u32,
            cache_creation: self.total_cache_creation.load(Ordering::Relaxed) as u32,
            input: self.total_input.load(Ordering::Relaxed) as u32,
        }
    }

    /// 获取缓存命中率
    pub fn get_hit_rate(&self) -> f64 {
        let metrics = self.get_cumulative_metrics();
        metrics.hit_rate()
    }

    /// 检查缓存健康状态
    pub fn check_health(&self) -> CacheHealthStatus {
        let metrics = self.get_cumulative_metrics();
        metrics.health_status()
    }

    /// 获取缓存断裂次数
    pub fn get_cache_breaks(&self) -> u32 {
        self.cache_breaks.load(Ordering::Relaxed)
    }

    /// 获取请求次数
    pub fn get_request_count(&self) -> u64 {
        self.request_count.load(Ordering::Relaxed)
    }

    /// 获取缓存断裂率
    pub fn get_break_rate(&self) -> f64 {
        let breaks = self.cache_breaks.load(Ordering::Relaxed) as f64;
        let requests = self.request_count.load(Ordering::Relaxed) as f64;

        if requests == 0.0 {
            0.0
        } else {
            breaks / requests
        }
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        self.total_cache_read.store(0, Ordering::Relaxed);
        self.total_cache_creation.store(0, Ordering::Relaxed);
        self.total_input.store(0, Ordering::Relaxed);
        self.cache_breaks.store(0, Ordering::Relaxed);
        self.request_count.store(0, Ordering::Relaxed);

        info!("Cache optimizer stats reset");
    }

    /// 生成缓存健康报告
    pub async fn generate_health_report(&self) -> CacheHealthReport {
        let metrics = self.get_cumulative_metrics();
        let hit_rate = metrics.hit_rate();
        let health = metrics.health_status();
        let breaks = self.get_cache_breaks();
        let break_rate = self.get_break_rate();

        CacheHealthReport {
            hit_rate,
            health_status: health,
            cache_breaks: breaks,
            break_rate,
            total_requests: self.get_request_count(),
            metrics,
        }
    }

    /// 获取当前 params 的缓存键
    pub async fn get_current_cache_key(&self) -> Option<String> {
        let params = self.current_params.read().await;
        params.as_ref().map(|p| p.cache_key())
    }

    /// 建议优化措施
    pub async fn suggest_optimizations(&self) -> Vec<String> {
        let mut suggestions = Vec::new();
        let health = self.check_health();

        match health {
            CacheHealthStatus::Poor => {
                suggestions
                    .push("缓存命中率低于 60%，请检查 system prompt 是否频繁变化".to_string());
                suggestions.push("考虑使用缓存预热策略".to_string());
            }
            CacheHealthStatus::Normal => {
                suggestions.push("缓存命中率正常，可尝试优化 message 顺序".to_string());
            }
            CacheHealthStatus::Good | CacheHealthStatus::Excellent => {
                suggestions.push("缓存状态优秀，保持当前配置".to_string());
            }
        }

        let break_rate = self.get_break_rate();
        if break_rate > 0.3 {
            suggestions.push(format!(
                "缓存断裂率{:.1}%较高，检查 fork context 一致性",
                break_rate * 100.0
            ));
        }

        suggestions
    }
}

impl Default for CacheOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

/// 缓存健康报告
pub struct CacheHealthReport {
    /// 缓存命中率
    pub hit_rate: f64,
    /// 健康状态
    pub health_status: CacheHealthStatus,
    /// 缓存断裂次数
    pub cache_breaks: u32,
    /// 断裂率
    pub break_rate: f64,
    /// 总请求数
    pub total_requests: u64,
    /// 详细指标
    pub metrics: CacheMetrics,
}

impl CacheHealthReport {
    /// 格式化为字符串
    pub fn to_string(&self) -> String {
        format!(
            "缓存健康报告:\n\
             - 命中率：{:.1}%\n\
             - 健康状态：{:?}\n\
             - 缓存断裂：{} 次 ({:.1}%)\n\
             - 总请求：{}\n\
             - 缓存读取：{} tokens\n\
             - 缓存创建：{} tokens\n\
             - 普通输入：{} tokens",
            self.hit_rate * 100.0,
            self.health_status,
            self.cache_breaks,
            self.break_rate * 100.0,
            self.total_requests,
            self.metrics.cache_read,
            self.metrics.cache_creation,
            self.metrics.input,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_metrics_hit_rate() {
        let metrics = CacheMetrics {
            cache_read: 800,
            cache_creation: 100,
            input: 100,
        };

        let hit_rate = metrics.hit_rate();
        assert!((hit_rate - 0.8).abs() < 0.001);

        let health = metrics.health_status();
        assert_eq!(health, CacheHealthStatus::Excellent);
    }

    #[test]
    fn test_cache_metrics_poor() {
        let metrics = CacheMetrics {
            cache_read: 100,
            cache_creation: 200,
            input: 700,
        };

        let hit_rate = metrics.hit_rate();
        assert!((hit_rate - 0.1).abs() < 0.001);

        let health = metrics.health_status();
        assert_eq!(health, CacheHealthStatus::Poor);
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
    async fn test_cache_optimizer_params_change() {
        use crate::forked_agent::CacheSafeParams;
        use crate::query_engine::Message;

        let optimizer = CacheOptimizer::new();

        let params1 = CacheSafeParams {
            system_prompt: "prompt1".to_string(),
            user_context: "user1".to_string(),
            system_context: "ctx".to_string(),
            tool_use_context: "tools".to_string(),
            fork_context_messages: vec![],
        };

        optimizer.update_params(params1).await;

        let params2 = CacheSafeParams {
            system_prompt: "prompt2".to_string(), // 变化
            user_context: "user1".to_string(),
            system_context: "ctx".to_string(),
            tool_use_context: "tools".to_string(),
            fork_context_messages: vec![],
        };

        optimizer.update_params(params2).await;

        // 应该记录一次缓存断裂
        assert_eq!(optimizer.get_cache_breaks(), 1);

        let break_rate = optimizer.get_break_rate();
        assert!((break_rate - 0.5).abs() < 0.01); // 2 次请求，1 次断裂
    }

    #[tokio::test]
    async fn test_cache_optimizer_health_report() {
        let optimizer = CacheOptimizer::new();

        // 记录一些用量
        for i in 0..10 {
            let usage = TokenUsage {
                input_tokens: 100,
                output_tokens: 50,
                cache_creation_input_tokens: if i < 2 { 100 } else { 0 },
                cache_read_input_tokens: if i >= 2 { 800 } else { 0 },
            };
            optimizer.record_usage(&usage);
        }

        let report = optimizer.generate_health_report().await;

        assert!(report.hit_rate > 0.7);
        assert_eq!(report.total_requests, 0); // 没有调用 update_params
        assert!(report.metrics.cache_read > 0);
    }

    #[tokio::test]
    async fn test_suggest_optimizations() {
        let optimizer = CacheOptimizer::new();

        // 记录低命中率的数据
        for _ in 0..10 {
            let usage = TokenUsage {
                input_tokens: 900,
                output_tokens: 50,
                cache_creation_input_tokens: 100,
                cache_read_input_tokens: 0,
            };
            optimizer.record_usage(&usage);
        }

        let suggestions = optimizer.suggest_optimizations().await;

        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.contains("低于 60%")));
    }
}
