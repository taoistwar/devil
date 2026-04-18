//! 成本追踪模块
//!
//! 实现：
//! - updateUsage: 处理单条流式事件的 usage 增量（>0 守卫）
//! - accumulateUsage: 跨消息累加总量
//! - CostTracker: 成本预算控制

use anyhow::Result;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Token 用量
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenUsage {
    /// 输入 token
    pub input_tokens: u32,
    /// 输出 token
    pub output_tokens: u32,
    /// 缓存创建 token
    pub cache_creation_input_tokens: u32,
    /// 缓存读取 token
    pub cache_read_input_tokens: u32,
}

impl TokenUsage {
    /// 创建新的用量记录
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取总输入 token（包含缓存）
    pub fn total_input(&self) -> u32 {
        self.input_tokens + self.cache_creation_input_tokens + self.cache_read_input_tokens
    }

    /// 获取总 token 数
    pub fn total(&self) -> u32 {
        self.total_input() + self.output_tokens
    }

    /// 估算成本（美元）
    pub fn estimate_cost(&self, model: &str) -> f64 {
        // 简化的定价模型
        let (input_price, output_price) = match model {
            "claude-3-opus" => (0.000_015, 0.000_075), // $15/$75 per 1M
            "claude-3-sonnet" => (0.000_003, 0.000_015), // $3/$15 per 1M
            "claude-3-haiku" => (0.000_00025, 0.000_00125), // $0.25/$1.25 per 1M
            _ => (0.000_003, 0.000_015),               // 默认 Sonnet 价格
        };

        // 缓存读取价格降低 90%
        let cache_read_price = input_price * 0.1;
        // 缓存创建价格增加 25%
        let cache_create_price = input_price * 1.25;

        let input_cost = self.input_tokens as f64 * input_price;
        let output_cost = self.output_tokens as f64 * output_price;
        let cache_create_cost = self.cache_creation_input_tokens as f64 * cache_create_price;
        let cache_read_cost = self.cache_read_input_tokens as f64 * cache_read_price;

        input_cost + output_cost + cache_create_cost + cache_read_cost
    }

    /// 合并另一个用量
    pub fn merge(&mut self, other: &TokenUsage) {
        accumulate_usage(self, other.clone());
    }
}

/// Usage 增量（流式事件）
#[derive(Debug, Clone, Default)]
pub struct UsageDelta {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    pub cache_creation_input_tokens: Option<u32>,
    pub cache_read_input_tokens: Option<u32>,
}

/// 更新用量（处理单条流式事件）
///
/// 使用 > 0 守卫防止真实值被覆盖为零
/// 例如：当 API 返回 input_tokens: 0 时，不应该将之前记录的真实值覆盖为零
pub fn update_usage(current: &mut TokenUsage, delta: UsageDelta) {
    // > 0 守卫：只有正增量才累加
    if let Some(val) = delta.input_tokens {
        if val > 0 {
            current.input_tokens = current.input_tokens.saturating_add(val);
            debug!("Updated input_tokens: +{} -> {}", val, current.input_tokens);
        }
    }

    if let Some(val) = delta.output_tokens {
        if val > 0 {
            current.output_tokens = current.output_tokens.saturating_add(val);
            debug!(
                "Updated output_tokens: +{} -> {}",
                val, current.output_tokens
            );
        }
    }

    if let Some(val) = delta.cache_creation_input_tokens {
        if val > 0 {
            current.cache_creation_input_tokens =
                current.cache_creation_input_tokens.saturating_add(val);
            debug!(
                "Updated cache_creation_input_tokens: +{} -> {}",
                val, current.cache_creation_input_tokens
            );
        }
    }

    if let Some(val) = delta.cache_read_input_tokens {
        if val > 0 {
            current.cache_read_input_tokens = current.cache_read_input_tokens.saturating_add(val);
            debug!(
                "Updated cache_read_input_tokens: +{} -> {}",
                val, current.cache_read_input_tokens
            );
        }
    }
}

/// 累加用量（跨消息）
///
/// 简单的字段求和，将各字段分别累加
pub fn accumulate_usage(total: &mut TokenUsage, addition: TokenUsage) {
    total.input_tokens = total.input_tokens.saturating_add(addition.input_tokens);
    total.output_tokens = total.output_tokens.saturating_add(addition.output_tokens);
    total.cache_creation_input_tokens = total
        .cache_creation_input_tokens
        .saturating_add(addition.cache_creation_input_tokens);
    total.cache_read_input_tokens = total
        .cache_read_input_tokens
        .saturating_add(addition.cache_read_input_tokens);

    debug!(
        "Accumulated usage: input={} output={} cache_create={} cache_read={}",
        total.input_tokens,
        total.output_tokens,
        total.cache_creation_input_tokens,
        total.cache_read_input_tokens
    );
}

/// 成本追踪器
pub struct CostTracker {
    /// 当前用量
    usage: Arc<RwLock<TokenUsage>>,
    /// 预算上限（美元）
    max_budget_usd: Arc<AtomicU64>, // 以美分存储
    /// 模型名称
    model: Arc<str>,
}

use tokio::sync::RwLock;

impl CostTracker {
    /// 创建新的成本追踪器
    pub fn new(model: String, max_budget_usd: Option<f64>) -> Self {
        let max_budget_cents = max_budget_usd
            .map(|b| (b * 100.0) as u64)
            .unwrap_or(u64::MAX);

        Self {
            usage: Arc::new(RwLock::new(TokenUsage::new())),
            max_budget_usd: Arc::new(AtomicU64::new(max_budget_cents)),
            model: model.into(),
        }
    }

    /// 更新用量
    pub async fn update_usage(&self, delta: UsageDelta) {
        let mut usage = self.usage.write().await;
        update_usage(&mut *usage, delta);
    }

    /// 获取当前用量
    pub async fn get_usage(&self) -> TokenUsage {
        self.usage.read().await.clone()
    }

    /// 检查是否超出预算
    pub async fn is_over_budget(&self) -> bool {
        let usage = self.usage.read().await;
        let current_cost_cents = (usage.estimate_cost(&self.model) * 100.0) as u64;
        let max_budget = self.max_budget_usd.load(Ordering::Relaxed);

        if current_cost_cents >= max_budget {
            warn!(
                "Budget exceeded: ${:.4} >= ${:.4}",
                current_cost_cents as f64 / 100.0,
                max_budget as f64 / 100.0
            );
            true
        } else {
            false
        }
    }

    /// 获取预算使用率
    pub async fn get_budget_usage(&self) -> f64 {
        let usage = self.usage.read().await;
        let current_cost = usage.estimate_cost(&self.model);
        let max_budget = self.max_budget_usd.load(Ordering::Relaxed) as f64 / 100.0;

        if max_budget == 0.0 || max_budget == f64::MAX {
            0.0
        } else {
            current_cost / max_budget
        }
    }

    /// 设置预算上限
    pub fn set_max_budget(&self, max_budget_usd: f64) {
        self.max_budget_usd
            .store((max_budget_usd * 100.0) as u64, Ordering::Relaxed);
        info!("Budget set to: ${:.2}", max_budget_usd);
    }

    /// 获取预算预警级别
    pub async fn get_budget_warning_level(&self) -> BudgetWarningLevel {
        let usage_ratio = self.get_budget_usage().await;

        if usage_ratio >= 1.0 {
            BudgetWarningLevel::Exceeded
        } else if usage_ratio >= 0.95 {
            BudgetWarningLevel::Critical
        } else if usage_ratio >= 0.80 {
            BudgetWarningLevel::Warning
        } else if usage_ratio >= 0.50 {
            BudgetWarningLevel::Notice
        } else {
            BudgetWarningLevel::Normal
        }
    }

    /// 重置用量
    pub async fn reset(&self) {
        let mut usage = self.usage.write().await;
        *usage = TokenUsage::new();
        info!("Cost tracker reset");
    }
}

/// 预算预警级别
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BudgetWarningLevel {
    /// < 50% - 正常运行
    Normal,
    /// 50%-80% - 轻量级提醒
    Notice,
    /// 80%-95% - 明确警告
    Warning,
    /// 95%-100% - 强烈警告
    Critical,
    /// >= 100% - 硬限制
    Exceeded,
}

/// 缓存命中率统计
#[derive(Debug, Clone)]
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_usage_positive() {
        let mut usage = TokenUsage::new();

        let delta = UsageDelta {
            input_tokens: Some(100),
            output_tokens: Some(50),
            cache_creation_input_tokens: Some(200),
            cache_read_input_tokens: Some(300),
        };

        update_usage(&mut usage, delta);

        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.cache_creation_input_tokens, 200);
        assert_eq!(usage.cache_read_input_tokens, 300);
    }

    #[test]
    fn test_update_usage_zero_guard() {
        let mut usage = TokenUsage::new();
        usage.input_tokens = 1000;

        // 零值不应该覆盖现有值
        let delta = UsageDelta {
            input_tokens: Some(0),
            ..Default::default()
        };

        update_usage(&mut usage, delta);

        assert_eq!(usage.input_tokens, 1000); // 保持不变
    }

    #[test]
    fn test_update_usage_none_guard() {
        let mut usage = TokenUsage::new();
        usage.input_tokens = 1000;

        // None 不应该影响现有值
        let delta = UsageDelta {
            input_tokens: None,
            ..Default::default()
        };

        update_usage(&mut usage, delta);

        assert_eq!(usage.input_tokens, 1000); // 保持不变
    }

    #[test]
    fn test_accumulate_usage() {
        let mut total = TokenUsage::new();
        total.input_tokens = 100;

        let addition = TokenUsage {
            input_tokens: 50,
            output_tokens: 30,
            ..Default::default()
        };

        accumulate_usage(&mut total, addition);

        assert_eq!(total.input_tokens, 150);
        assert_eq!(total.output_tokens, 30);
    }

    #[test]
    fn test_total_calculation() {
        let usage = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 200,
            cache_read_input_tokens: 300,
        };

        assert_eq!(usage.total_input(), 600);
        assert_eq!(usage.total(), 650);
    }

    #[test]
    fn test_estimate_cost() {
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let cost = usage.estimate_cost("claude-3-sonnet");

        // 输入：1000 * $0.000003 = $0.003
        // 输出：500 * $0.000015 = $0.0075
        // 总计：$0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cache_metrics_hit_rate() {
        let metrics = CacheMetrics {
            cache_read: 800,
            cache_creation: 200,
            input: 500,
        };

        // 命中率 = 800 / (800 + 200 + 500) = 800 / 1500 = 0.533
        let hit_rate = metrics.hit_rate();
        assert!((hit_rate - 0.533).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_cost_tracker_budget() {
        let tracker = CostTracker::new("claude-3-sonnet".to_string(), Some(10.0));

        // 初始预算使用率应该为 0
        let usage = tracker.get_budget_usage().await;
        assert_eq!(usage, 0.0);

        // 模拟一些用量
        tracker
            .update_usage(UsageDelta {
                input_tokens: Some(100000),
                output_tokens: Some(50000),
                ..Default::default()
            })
            .await;

        let usage = tracker.get_budget_usage().await;
        assert!(usage > 0.0 && usage < 1.0);
    }

    #[tokio::test]
    async fn test_budget_warning_levels() {
        let tracker = CostTracker::new("claude-3-sonnet".to_string(), Some(1.0));

        // 初始应该是 Normal
        let level = tracker.get_budget_warning_level().await;
        assert_eq!(level, BudgetWarningLevel::Normal);
    }
}
