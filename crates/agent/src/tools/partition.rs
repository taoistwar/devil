//! 并发分区模块
//! 
//! 实现工具调度的并发分区策略：
//! - 将工具调用按顺序划分为批次
//! - 并发安全批次并行执行
//! - 非安全批次串行执行

use crate::tools::tool::ToolUseBlock;

/// 工具调用批次
#[derive(Debug, Clone)]
pub struct ToolCallBatch {
    /// 批次中的工具调用
    pub calls: Vec<ToolUseBlock>,
    /// 是否为并发安全批次
    pub is_concurrency_safe: bool,
}

impl ToolCallBatch {
    pub fn new(calls: Vec<ToolUseBlock>, is_concurrency_safe: bool) -> Self {
        Self {
            calls,
            is_concurrency_safe,
        }
    }
}

/// 并发分区器
/// 
/// 负责将工具调用序列划分为并发安全的批次
pub struct ConcurrentPartitioner {
    /// 最大并发度
    max_concurrency: usize,
}

impl ConcurrentPartitioner {
    /// 创建新的分区器
    pub fn new(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }

    /// 创建默认分区器
    pub fn with_defaults() -> Self {
        // 默认并发度为 10，与环境变量控制一致
        Self::new(10)
    }

    /// 执行并发分区
    /// 
    /// 分区算法：
    /// 1. 遍历所有工具调用
    /// 2. 检查每个工具的并发安全属性
    /// 3. 如果当前工具安全且前一个批次也安全，则合并到同一批次
    /// 4. 否则开启新批次
    /// 
    /// 这个算法确保：
    /// - 连续的并发安全工具会被分到同一批次并行执行
    /// - 非安全工具总是独占一个批次串行执行
    /// - 批次顺序与原始调用顺序一致
    pub fn partition(&self, calls: Vec<ToolUseCallInfo>) -> Vec<ToolCallBatch> {
        if calls.is_empty() {
            return Vec::new();
        }

        let mut batches: Vec<ToolCallBatch> = Vec::new();
        let mut current_batch_calls: Vec<ToolUseBlock> = Vec::new();
        let mut current_batch_safe = true;

        for call in calls {
            // 检查当前工具是否安全
            let is_safe = call.concurrency_safe;

            // 决定是否需要开启新批次
            let need_new_batch = if is_safe {
                // 当前工具安全：只有当前批次不安全时才需要新批次
                !current_batch_safe
            } else {
                // 当前工具不安全：如果当前批次已有工具，需要新批次
                !current_batch_calls.is_empty()
            };

            if need_new_batch {
                // 提交当前批次
                if !current_batch_calls.is_empty() {
                    batches.push(ToolCallBatch::new(
                        current_batch_calls,
                        current_batch_safe,
                    ));
                }
                
                // 开始新批次
                current_batch_calls = Vec::new();
                current_batch_safe = is_safe;
            }

            // 添加到当前批次
            current_batch_calls.push(call.block);
            
            // 如果当前工具不安全，批次整体标记为不安全
            if !is_safe {
                current_batch_safe = false;
            }
        }

        // 提交最后一个批次
        if !current_batch_calls.is_empty() {
            batches.push(ToolCallBatch::new(
                current_batch_calls,
                current_batch_safe,
            ));
        }

        batches
    }

    /// 获取最大并发度
    pub fn max_concurrency(&self) -> usize {
        self.max_concurrency
    }
}

impl Default for ConcurrentPartitioner {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 工具调用信息
#[derive(Debug, Clone)]
pub struct ToolUseCallInfo {
    /// 工具调用块
    pub block: ToolUseBlock,
    /// 是否并发安全
    pub concurrency_safe: bool,
}

impl ToolUseCallInfo {
    pub fn new(block: ToolUseBlock, concurrency_safe: bool) -> Self {
        Self {
            block,
            concurrency_safe,
        }
    }
}

/// 并发执行结果
#[derive(Debug)]
pub struct BatchExecutionResult {
    /// 批次索引
    pub batch_index: usize,
    /// 各工具调用结果
    pub results: Vec<ToolCallResult>,
    /// 批次执行耗时（毫秒）
    pub duration_ms: u64,
}

/// 工具调用结果
#[derive(Debug)]
pub struct ToolCallResult {
    /// 工具调用 ID
    pub tool_use_id: String,
    /// 是否成功
    pub success: bool,
    /// 结果内容
    pub content: String,
    /// 错误信息
    pub error: Option<String>,
}

/// 分区策略说明
/// 
/// ```text
/// 示例：
/// 输入序列：[Read(a), Read(b), Bash(ls), Read(c)]
/// 
/// 分区结果：
/// Batch 1 (并发安全): [Read(a), Read(b)]   -- 并行执行
/// Batch 2 (非安全):   [Bash(ls)]            -- 串行执行  
/// Batch 3 (并发安全): [Read(c)]             -- 可并行（但只有一个工具）
/// 
/// 为什么 Read(c) 不能和 Bash(ls) 放在同一个批次？
/// 因为 Bash 命令可能有副作用——它可能创建新文件、修改文件内容或改变目录结构。
/// 如果在 Bash 执行的同时读取文件，Read 可能读到执行前的旧数据或执行后的新数据，
/// 导致不可预测的行为。串行执行确保了 Read(c) 看到的是 Bash(ls) 执行完成后的确定状态。
/// ```
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition_empty() {
        let partitioner = ConcurrentPartitioner::with_defaults();
        let batches = partitioner.partition(Vec::new());
        assert!(batches.is_empty());
    }

    #[test]
    fn test_partition_all_safe() {
        let partitioner = ConcurrentPartitioner::with_defaults();
        
        let calls = vec![
            ToolUseCallInfo::new(
                ToolUseBlock::new("1", "read", serde_json::json!({"path": "a.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("2", "read", serde_json::json!({"path": "b.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("3", "glob", serde_json::json!({"pattern": "*.ts"})),
                true,
            ),
        ];

        let batches = partitioner.partition(calls);
        assert_eq!(batches.len(), 1);
        assert!(batches[0].is_concurrency_safe);
        assert_eq!(batches[0].calls.len(), 3);
    }

    #[test]
    fn test_partition_all_unsafe() {
        let partitioner = ConcurrentPartitioner::with_defaults();
        
        let calls = vec![
            ToolUseCallInfo::new(
                ToolUseBlock::new("1", "bash", serde_json::json!({"command": "ls"})),
                false,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("2", "edit", serde_json::json!({"path": "a.ts"})),
                false,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("3", "write", serde_json::json!({"path": "b.ts"})),
                false,
            ),
        ];

        let batches = partitioner.partition(calls);
        assert_eq!(batches.len(), 3);
        for batch in &batches {
            assert!(!batch.is_concurrency_safe);
            assert_eq!(batch.calls.len(), 1);
        }
    }

    #[test]
    fn test_partition_mixed() {
        let partitioner = ConcurrentPartitioner::with_defaults();
        
        // 模拟示例：[Read(a), Read(b), Bash(ls), Read(c)]
        let calls = vec![
            ToolUseCallInfo::new(
                ToolUseBlock::new("1", "read", serde_json::json!({"path": "a.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("2", "read", serde_json::json!({"path": "b.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("3", "bash", serde_json::json!({"command": "ls"})),
                false,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("4", "read", serde_json::json!({"path": "c.ts"})),
                true,
            ),
        ];

        let batches = partitioner.partition(calls);
        
        // 期望结果：
        // Batch 1: [Read(a), Read(b)] - 并发安全
        // Batch 2: [Bash(ls)] - 非安全
        // Batch 3: [Read(c)] - 并发安全
        assert_eq!(batches.len(), 3);
        
        assert!(batches[0].is_concurrency_safe);
        assert_eq!(batches[0].calls.len(), 2);
        assert_eq!(batches[0].calls[0].name, "read");
        assert_eq!(batches[0].calls[1].name, "read");
        
        assert!(!batches[1].is_concurrency_safe);
        assert_eq!(batches[1].calls.len(), 1);
        assert_eq!(batches[1].calls[0].name, "bash");
        
        assert!(batches[2].is_concurrency_safe);
        assert_eq!(batches[2].calls.len(), 1);
        assert_eq!(batches[2].calls[0].name, "read");
    }

    #[test]
    fn test_partition_complex_sequence() {
        let partitioner = ConcurrentPartitioner::with_defaults();
        
        // 复杂序列：[Glob(*.ts), Grep(pattern), Bash(npm test), Read(a.ts), Edit(a.ts), Glob(*.json)]
        let calls = vec![
            ToolUseCallInfo::new(
                ToolUseBlock::new("1", "glob", serde_json::json!({"pattern": "*.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("2", "grep", serde_json::json!({"pattern": "TODO"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("3", "bash", serde_json::json!({"command": "npm test"})),
                false,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("4", "read", serde_json::json!({"path": "a.ts"})),
                true,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("5", "edit", serde_json::json!({"path": "a.ts"})),
                false,
            ),
            ToolUseCallInfo::new(
                ToolUseBlock::new("6", "glob", serde_json::json!({"pattern": "*.json"})),
                true,
            ),
        ];

        let batches = partitioner.partition(calls);
        
        // 期望结果：
        // Batch 1: [Glob, Grep] - 并发安全
        // Batch 2: [Bash] - 非安全
        // Batch 3: [Read] - 并发安全（但必须等 Bash 完成）
        // Batch 4: [Edit] - 非安全
        // Batch 5: [Glob] - 并发安全
        assert_eq!(batches.len(), 5);
        
        assert!(batches[0].is_concurrency_safe);
        assert_eq!(batches[0].calls.len(), 2);
        
        assert!(!batches[1].is_concurrency_safe);
        assert_eq!(batches[1].calls.len(), 1);
        
        assert!(batches[2].is_concurrency_safe);
        assert_eq!(batches[2].calls.len(), 1);
        
        assert!(!batches[3].is_concurrency_safe);
        assert_eq!(batches[3].calls.len(), 1);
        
        assert!(batches[4].is_concurrency_safe);
        assert_eq!(batches[4].calls.len(), 1);
    }
}
