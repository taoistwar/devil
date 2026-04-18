//! Token 预算追踪模块
//!
//! 实现压缩后的令牌预算控制和预警系统

use serde::{Deserialize, Serialize};

/// 压缩后最大恢复文件数
pub const POST_COMPACT_MAX_FILES_TO_RESTORE: u32 = 5;

/// 压缩后总令牌预算
pub const POST_COMPACT_TOKEN_BUDGET: u32 = 50_000;

/// 每个文件的令牌上限
pub const POST_COMPACT_MAX_TOKENS_PER_FILE: u32 = 5_000;

/// 每个技能的令牌上限
pub const POST_COMPACT_MAX_TOKENS_PER_SKILL: u32 = 5_000;

/// 技能独立预算
pub const POST_COMPACT_SKILLS_TOKEN_BUDGET: u32 = 25_000;

/// 令牌预算配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetConfig {
    /// 总预算
    pub total_budget: u32,
    /// 每文件预算
    pub per_file_budget: u32,
    /// 每技能预算
    pub per_skill_budget: u32,
    /// 技能总预算
    pub skills_total_budget: u32,
    /// 最大恢复文件数
    pub max_files_to_restore: u32,
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self {
            total_budget: POST_COMPACT_TOKEN_BUDGET,
            per_file_budget: POST_COMPACT_MAX_TOKENS_PER_FILE,
            per_skill_budget: POST_COMPACT_MAX_TOKENS_PER_SKILL,
            skills_total_budget: POST_COMPACT_SKILLS_TOKEN_BUDGET,
            max_files_to_restore: POST_COMPACT_MAX_FILES_TO_RESTORE,
        }
    }
}

/// 文件令牌预算
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTokenBudget {
    /// 文件路径
    pub path: String,
    /// 估计令牌数
    pub estimated_tokens: u32,
    /// 是否超出预算
    pub exceeds_budget: bool,
}

/// 令牌预算追踪器
///
/// # 反模式警告
///
/// 常见的错误是在压缩后立即重新加载所有之前读取的文件。
/// 这样做会迅速耗尽令牌预算，导致在几轮对话后再次触发压缩，
/// 形成"压缩 - 膨胀 - 再压缩"的恶性循环。
///
/// 正确做法是只重新加载当前任务需要的文件。
pub struct TokenBudgetTracker {
    config: TokenBudgetConfig,
    /// 当前使用的令牌数
    current_usage: u32,
    /// 已恢复的文件列表
    restored_files: Vec<FileTokenBudget>,
    /// 已使用的技能令牌数
    skills_usage: u32,
}

impl TokenBudgetTracker {
    /// 创建新的预算追踪器
    pub fn new(config: TokenBudgetConfig) -> Self {
        Self {
            config,
            current_usage: 0,
            restored_files: Vec::new(),
            skills_usage: 0,
        }
    }

    /// 创建默认配置的追踪器
    pub fn default() -> Self {
        Self::new(TokenBudgetConfig::default())
    }

    /// 尝试添加文件到恢复列表
    ///
    /// # 返回
    ///
    /// - `Ok(true)` - 文件已添加
    /// - `Ok(false)` - 文件超出预算，未添加
    /// - `Err(String)` - 错误信息
    pub fn try_restore_file(
        &mut self,
        path: impl Into<String>,
        estimated_tokens: u32,
    ) -> Result<bool, String> {
        // 检查文件数量限制
        if self.restored_files.len() >= self.config.max_files_to_restore as usize {
            return Ok(false);
        }

        // 检查每文件预算
        if estimated_tokens > self.config.per_file_budget {
            return Err(format!(
                "File exceeds per-file budget: {} > {}",
                estimated_tokens, self.config.per_file_budget
            ));
        }

        // 检查总预算
        let new_total = self.current_usage + estimated_tokens;
        if new_total > self.config.total_budget {
            return Ok(false);
        }

        // 添加文件
        self.restored_files.push(FileTokenBudget {
            path: path.into(),
            estimated_tokens,
            exceeds_budget: false,
        });
        self.current_usage += estimated_tokens;

        Ok(true)
    }

    /// 尝试使用技能令牌
    pub fn try_use_skill(
        &mut self,
        skill_name: &str,
        estimated_tokens: u32,
    ) -> Result<bool, String> {
        // 检查每技能预算
        if estimated_tokens > self.config.per_skill_budget {
            return Err(format!(
                "Skill '{}' exceeds per-skill budget: {} > {}",
                skill_name, estimated_tokens, self.config.per_skill_budget
            ));
        }

        // 检查技能总预算
        let new_skills_total = self.skills_usage + estimated_tokens;
        if new_skills_total > self.config.skills_total_budget {
            return Ok(false);
        }

        // 检查总预算
        let new_total = self.current_usage + estimated_tokens;
        if new_total > self.config.total_budget {
            return Ok(false);
        }

        self.skills_usage += estimated_tokens;
        self.current_usage += estimated_tokens;

        Ok(true)
    }

    /// 获取当前预算使用情况
    pub fn get_usage_stats(&self) -> TokenBudgetStats {
        TokenBudgetStats {
            current_usage: self.current_usage,
            total_budget: self.config.total_budget,
            usage_percentage: (self.current_usage as f32 / self.config.total_budget as f32) * 100.0,
            restored_files_count: self.restored_files.len() as u32,
            max_restored_files: self.config.max_files_to_restore,
            skills_usage: self.skills_usage,
            skills_budget: self.config.skills_total_budget,
        }
    }

    /// 判断是否接近预算上限
    pub fn is_near_budget_limit(&self, threshold: f32) -> bool {
        let usage_ratio = self.current_usage as f32 / self.config.total_budget as f32;
        usage_ratio >= threshold
    }

    /// 重置预算追踪
    pub fn reset(&mut self) {
        self.current_usage = 0;
        self.restored_files.clear();
        self.skills_usage = 0;
    }

    /// 获取已恢复的文件列表
    pub fn get_restored_files(&self) -> &[FileTokenBudget] {
        &self.restored_files
    }
}

/// 令牌预算统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudgetStats {
    /// 当前使用量
    pub current_usage: u32,
    /// 总预算
    pub total_budget: u32,
    /// 使用百分比
    pub usage_percentage: f32,
    /// 已恢复文件数
    pub restored_files_count: u32,
    /// 最大恢复文件数
    pub max_restored_files: u32,
    /// 技能使用量
    pub skills_usage: u32,
    /// 技能预算
    pub skills_budget: u32,
}

impl TokenBudgetStats {
    /// 判断是否健康（使用率 < 50%）
    pub fn is_healthy(&self) -> bool {
        self.usage_percentage < 50.0
    }

    /// 判断是否警告（使用率 50-80%）
    pub fn is_warning(&self) -> bool {
        self.usage_percentage >= 50.0 && self.usage_percentage < 80.0
    }

    /// 判断是否危险（使用率 >= 80%）
    pub fn is_danger(&self) -> bool {
        self.usage_percentage >= 80.0
    }
}

/// 压缩后令牌计数
///
/// `truePostCompactTokenCount` 是对压缩后上下文实际大小的估算
/// 它用于判断压缩是否会立即在下一轮触发再次压缩
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TruePostCompactTokenCount {
    /// 边界标记令牌数
    pub boundary_marker_tokens: u32,
    /// 摘要消息令牌数
    pub summary_tokens: u32,
    /// 附件令牌数
    pub attachments_tokens: u32,
    /// 钩子结果令牌数
    pub hook_results_tokens: u32,
    /// 总计
    pub total: u32,
}

impl TruePostCompactTokenCount {
    /// 计算总令牌数
    pub fn calculate(
        boundary_marker: u32,
        summary: u32,
        attachments: u32,
        hook_results: u32,
    ) -> Self {
        Self {
            boundary_marker_tokens: boundary_marker,
            summary_tokens: summary,
            attachments_tokens: attachments,
            hook_results_tokens: hook_results,
            total: boundary_marker + summary + attachments + hook_results,
        }
    }

    /// 判断是否会立即触发再次压缩
    pub fn would_trigger_auto_compact(&self, auto_compact_threshold: u32) -> bool {
        self.total >= auto_compact_threshold
    }
}

/// 令牌估算 Trait
pub trait TokenEstimator {
    /// 估算字符串的令牌数
    fn estimate_tokens(&self, text: &str) -> u32;
}

/// 简单令牌估算器（4 字符 ≈ 1 令牌）
pub struct SimpleTokenEstimator;

impl TokenEstimator for SimpleTokenEstimator {
    fn estimate_tokens(&self, text: &str) -> u32 {
        (text.len() as f32 / 4.0).ceil() as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_budget_constants() {
        assert_eq!(POST_COMPACT_TOKEN_BUDGET, 50_000);
        assert_eq!(POST_COMPACT_MAX_TOKENS_PER_FILE, 5_000);
        assert_eq!(POST_COMPACT_MAX_FILES_TO_RESTORE, 5);
    }

    #[test]
    fn test_try_restore_file_within_budget() {
        let mut tracker = TokenBudgetTracker::default();

        // 添加一个 3000 令牌的文件
        let result = tracker.try_restore_file("test.rs", 3000);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let stats = tracker.get_usage_stats();
        assert_eq!(stats.current_usage, 3000);
        assert_eq!(stats.restored_files_count, 1);
    }

    #[test]
    fn test_try_restore_file_exceeds_budget() {
        let mut tracker = TokenBudgetTracker::default();

        // 添加一个超出预算的文件（6000 > 5000）
        let result = tracker.try_restore_file("large.rs", 6000);
        assert!(result.is_err());

        // 添加刚好在预算内的文件
        let result = tracker.try_restore_file("test.rs", 5000);
        assert!(result.is_ok());
    }

    #[test]
    fn test_try_restore_multiple_files() {
        let mut tracker = TokenBudgetTracker::default();

        // 添加 5 个文件
        for i in 0..5 {
            let result = tracker.try_restore_file(format!("file{}.rs", i), 1000);
            assert!(result.is_ok());
            assert!(result.unwrap());
        }

        // 第 6 个文件应该被拒绝（超过最大数量）
        let result = tracker.try_restore_file("file6.rs", 1000);
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_skill_budget() {
        let mut tracker = TokenBudgetTracker::default();

        // 使用一个技能
        let result = tracker.try_use_skill("test_skill", 4000);
        assert!(result.is_ok());
        assert!(result.unwrap());

        let stats = tracker.get_usage_stats();
        assert_eq!(stats.skills_usage, 4000);

        // 超出技能预算
        let result = tracker.try_use_skill("large_skill", 6000);
        assert!(result.is_err());
    }

    #[test]
    fn test_budget_stats_health() {
        let mut tracker = TokenBudgetTracker::default();

        // 健康状态
        tracker.try_restore_file("small.rs", 10000);
        let stats = tracker.get_usage_stats();
        assert!(stats.is_healthy());

        // 警告状态
        tracker.reset();
        tracker.try_restore_file("medium.rs", 30000);
        let stats = tracker.get_usage_stats();
        assert!(stats.is_warning());

        // 危险状态
        tracker.reset();
        tracker.try_restore_file("large.rs", 45000);
        let stats = tracker.get_usage_stats();
        assert!(stats.is_danger());
    }

    #[test]
    fn test_true_post_compact_tokens() {
        let count = TruePostCompactTokenCount::calculate(100, 10000, 2000, 500);

        assert_eq!(count.total, 12600);
        assert!(!count.would_trigger_auto_compact(200000));
        assert!(count.would_trigger_auto_compact(10000));
    }

    #[test]
    fn test_simple_token_estimator() {
        let estimator = SimpleTokenEstimator;

        // 100 字符 ≈ 25 令牌
        let tokens = estimator.estimate_tokens(&"a".repeat(100));
        assert!(tokens >= 24 && tokens <= 26);
    }
}
