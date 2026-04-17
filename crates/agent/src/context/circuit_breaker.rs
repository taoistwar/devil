//! 断路器模式实现
//! 
//! 保护系统免受级联失败影响
//! 连续 3 次压缩失败后停止尝试

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// 断路器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    /// 闭合状态：正常工作
    Closed,
    /// 断开状态：熔断，不再尝试
    Open,
    /// 半开状态：试探性尝试
    HalfOpen,
}

impl CircuitBreakerState {
    /// 判断是否允许执行
    pub fn allows_execution(&self) -> bool {
        matches!(self, CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen)
    }
}

/// 断路器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// 连续失败次数阈值
    pub failure_threshold: u32,
    /// 半开状态等待时间
    pub half_open_wait_secs: u64,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            // Claude Code 默认 3 次
            failure_threshold: 3,
            // 半开状态等待 60 秒
            half_open_wait_secs: 60,
        }
    }
}

/// 断路器实现
/// 
/// # 设计动机
/// 
/// 当连续失败次数达到阈值时，系统直接跳过后续的压缩尝试。
/// 如果盲目重试，系统会在每个轮次都发起注定失败的 API 调用，浪费大量资源。
/// 
/// # 真实数据
/// 
/// 引入断路器前曾观察到 1,279 个会话出现 50 次以上的连续压缩失败（最高达 3,272 次），
/// 每天浪费约 250K 次 API 调用。引入后，这类级联失败被彻底消除。
pub struct CircuitBreaker {
    /// 当前状态
    state: Arc<RwLock<CircuitBreakerState>>,
    /// 连续失败计数器
    consecutive_failures: Arc<AtomicU32>,
    /// 上次失败时间
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    /// 配置
    config: CircuitBreakerConfig,
    /// 名称（用于日志）
    name: String,
}

impl CircuitBreaker {
    /// 创建新的断路器
    pub fn new(name: impl Into<String>) -> Self {
        Self::with_config(name, CircuitBreakerConfig::default())
    }

    /// 创建带配置的断路器
    pub fn with_config(name: impl Into<String>, config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            consecutive_failures: Arc::new(AtomicU32::new(0)),
            last_failure_time: Arc::new(RwLock::new(None)),
            config,
            name: name.into(),
        }
    }

    /// 获取当前状态
    pub fn get_state(&self) -> CircuitBreakerState {
        *self.state.read().unwrap()
    }

    /// 判断是否允许执行
    pub fn allows_execution(&self) -> bool {
        let state = self.get_state();
        
        match state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => {
                // 检查是否可以进入半开状态
                if let Some(last_failure) = *self.last_failure_time.read().unwrap() {
                    let elapsed = last_failure.elapsed();
                    if elapsed >= Duration::from_secs(self.config.half_open_wait_secs) {
                        // 进入半开状态
                        self.set_state(CircuitBreakerState::HalfOpen);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitBreakerState::HalfOpen => true,
        }
    }

    /// 记录成功
    /// 
    /// 成功时，失败计数器重置为零，断路器回到闭合状态
    pub fn record_success(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        self.set_state(CircuitBreakerState::Closed);
        
        tracing::debug!(
            "[{}] Circuit breaker reset to Closed (success after {} failures)",
            self.name,
            self.consecutive_failures.load(Ordering::Relaxed)
        );
    }

    /// 记录失败
    /// 
    /// 失败时，计数器递增。连续失败达到阈值后，断路器进入断开状态
    pub fn record_failure(&self) {
        let failures = self.consecutive_failures.fetch_add(1, Ordering::SeqCst) + 1;
        
        *self.last_failure_time.write().unwrap() = Some(Instant::now());
        
        if failures >= self.config.failure_threshold {
            let old_state = self.get_state();
            if old_state != CircuitBreakerState::Open {
                self.set_state(CircuitBreakerState::Open);
                
                tracing::warn!(
                    "[{}] Circuit breaker OPENED after {} consecutive failures",
                    self.name,
                    failures
                );
            }
        } else {
            tracing::debug!(
                "[{}] Recorded failure {}/{}",
                self.name,
                failures,
                self.config.failure_threshold
            );
        }
    }

    /// 获取连续失败次数
    pub fn get_consecutive_failures(&self) -> u32 {
        self.consecutive_failures.load(Ordering::Relaxed)
    }

    /// 手动重置断路器
    pub fn reset(&self) {
        self.consecutive_failures.store(0, Ordering::SeqCst);
        *self.last_failure_time.write().unwrap() = None;
        self.set_state(CircuitBreakerState::Closed);
        
        tracing::info!("[{}] Circuit breaker manually reset", self.name);
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.get_state(),
            consecutive_failures: self.get_consecutive_failures(),
            failure_threshold: self.config.failure_threshold,
            half_open_wait_secs: self.config.half_open_wait_secs,
        }
    }

    /// 设置状态（内部使用）
    fn set_state(&self, state: CircuitBreakerState) {
        *self.state.write().unwrap() = state;
    }
}

impl Clone for CircuitBreaker {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            consecutive_failures: self.consecutive_failures.clone(),
            last_failure_time: self.last_failure_time.clone(),
            config: self.config.clone(),
            name: self.name.clone(),
        }
    }
}

/// 断路器统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerStats {
    /// 当前状态
    pub state: CircuitBreakerState,
    /// 连续失败次数
    pub consecutive_failures: u32,
    /// 失败阈值
    pub failure_threshold: u32,
    /// 半开等待时间（秒）
    pub half_open_wait_secs: u64,
}

/// 压缩结果（用于断路器判断）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionAttemptResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error_message: Option<String>,
    /// 是否 prompt_too_long 错误
    pub is_prompt_too_long: bool,
}

impl CompactionAttemptResult {
    /// 创建成功结果
    pub fn success() -> Self {
        Self {
            success: true,
            error_message: None,
            is_prompt_too_long: false,
        }
    }

    /// 创建失败结果
    pub fn failure(error_message: impl Into<String>) -> Self {
        let error = error_message.into();
        let is_prompt_too_long = error.to_lowercase().contains("prompt_too_long")
            || error.to_lowercase().contains("context window");
        
        Self {
            success: false,
            error_message: Some(error),
            is_prompt_too_long,
        }
    }
}

/// 处理压缩尝试结果，更新断路器状态
pub fn handle_compaction_result(
    circuit_breaker: &CircuitBreaker,
    result: &CompactionAttemptResult,
) -> bool {
    if result.success {
        circuit_breaker.record_success();
        true
    } else {
        circuit_breaker.record_failure();
        
        // 如果是 prompt_too_long 错误，可能意味着上下文不可压缩
        if result.is_prompt_too_long {
            tracing::warn!(
                "[Compaction] Prompt too long error - context may be irrecoverable"
            );
        }
        
        false
    }
}

/// 检查是否应该跳过自动压缩（断路器保护）
pub fn should_skip_auto_compact(circuit_breaker: &CircuitBreaker) -> bool {
    !circuit_breaker.allows_execution()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new("test");
        assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
        assert_eq!(cb.get_consecutive_failures(), 0);
        assert!(cb.allows_execution());
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new("test");
        
        // 记录 3 次失败
        cb.record_failure();
        cb.record_failure();
        cb.record_failure();
        
        // 应该打开
        assert_eq!(cb.get_state(), CircuitBreakerState::Open);
        assert!(!cb.allows_execution());
        assert_eq!(cb.get_consecutive_failures(), 3);
    }

    #[test]
    fn test_circuit_breaker_resets_on_success() {
        let cb = CircuitBreaker::new("test");
        
        // 记录 2 次失败
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_consecutive_failures(), 2);
        
        // 记录成功
        cb.record_success();
        
        // 应该重置
        assert_eq!(cb.get_state(), CircuitBreakerState::Closed);
        assert_eq!(cb.get_consecutive_failures(), 0);
    }

    #[test]
    fn test_compaction_result_parsing() {
        let success = CompactionAttemptResult::success();
        assert!(success.success);
        assert!(success.error_message.is_none());
        
        let failure = CompactionAttemptResult::failure("Network error");
        assert!(!failure.success);
        assert!(failure.error_message.is_some());
        assert!(!failure.is_prompt_too_long);
        
        let prompt_failure = CompactionAttemptResult::failure("prompt_too_long");
        assert!(!prompt_failure.success);
        assert!(prompt_failure.is_prompt_too_long);
    }

    #[test]
    fn test_handle_compaction_result() {
        let cb = CircuitBreaker::new("test");
        
        // 成功
        let result = CompactionAttemptResult::success();
        assert!(handle_compaction_result(&cb, &result));
        assert_eq!(cb.get_consecutive_failures(), 0);
        
        // 失败
        let result = CompactionAttemptResult::failure("Error");
        assert!(!handle_compaction_result(&cb, &result));
        assert_eq!(cb.get_consecutive_failures(), 1);
    }

    #[test]
    fn test_hal_open_transition() {
        let cb = CircuitBreaker::with_config(
            "test",
            CircuitBreakerConfig {
                failure_threshold: 2,
                half_open_wait_secs: 1, // 1 秒用于测试
            },
        );
        
        // 记录 2 次失败，进入 Open
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.get_state(), CircuitBreakerState::Open);
        
        // 等待超过半开时间
        std::thread::sleep(Duration::from_secs(2));
        
        // 现在应该允许执行（进入 HalfOpen）
        assert!(cb.allows_execution());
        assert_eq!(cb.get_state(), CircuitBreakerState::HalfOpen);
    }
}
