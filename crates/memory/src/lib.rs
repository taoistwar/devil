//! Memory crate - 提供数据存储和记忆功能
//! 
//! 本 crate 负责管理 Agent 的短期和长期记忆存储

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// 记忆条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// 记忆 ID
    pub id: String,
    /// 记忆内容
    pub content: String,
    /// 时间戳
    pub timestamp: u64,
    /// 标签
    pub tags: Vec<String>,
}

/// 短期记忆存储
pub struct ShortTermMemory {
    entries: RwLock<HashMap<String, MemoryEntry>>,
    max_capacity: usize,
}

impl ShortTermMemory {
    /// 创建新的短期记忆存储
    pub fn new(max_capacity: usize) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            max_capacity,
        }
    }

    /// 添加记忆条目
    pub async fn add(&self, entry: MemoryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        
        // 如果超出容量，移除最旧的条目
        if entries.len() >= self.max_capacity {
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, v)| v.timestamp)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
                debug!("Removed oldest memory entry: {}", oldest_key);
            }
        }
        
        let id = entry.id.clone();
        entries.insert(id.clone(), entry);
        info!("Added short-term memory: {}", id);
        Ok(())
    }

    /// 获取记忆条目
    pub async fn get(&self, id: &str) -> Option<MemoryEntry> {
        let entries = self.entries.read().await;
        entries.get(id).cloned()
    }

    /// 删除记忆条目
    pub async fn remove(&self, id: &str) -> Result<()> {
        let mut entries = self.entries.write().await;
        entries.remove(id);
        debug!("Removed memory entry: {}", id);
        Ok(())
    }

    /// 获取所有记忆
    pub async fn get_all(&self) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        entries.values().cloned().collect()
    }

    /// 清空所有记忆
    pub async fn clear(&self) {
        let mut entries = self.entries.write().await;
        entries.clear();
        info!("Cleared all short-term memory");
    }
}

impl Default for ShortTermMemory {
    fn default() -> Self {
        Self::new(100)
    }
}

/// 长期记忆存储（简化版本）
pub struct LongTermMemory {
    entries: RwLock<HashMap<String, MemoryEntry>>,
}

impl LongTermMemory {
    /// 创建新的长期记忆存储
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    /// 存储记忆
    pub async fn store(&self, entry: MemoryEntry) -> Result<()> {
        let mut entries = self.entries.write().await;
        let id = entry.id.clone();
        entries.insert(id.clone(), entry);
        info!("Stored long-term memory: {}", id);
        Ok(())
    }

    /// 检索记忆
    pub async fn retrieve(&self, id: &str) -> Option<MemoryEntry> {
        let entries = self.entries.read().await;
        entries.get(id).cloned()
    }

    /// 按标签搜索记忆
    pub async fn search_by_tags(&self, tags: &[String]) -> Vec<MemoryEntry> {
        let entries = self.entries.read().await;
        entries
            .values()
            .filter(|entry| {
                entry.tags.iter().any(|tag| tags.contains(tag))
            })
            .cloned()
            .collect()
    }
}

impl Default for LongTermMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// 记忆管理器
pub struct MemoryManager {
    short_term: ShortTermMemory,
    long_term: LongTermMemory,
}

impl MemoryManager {
    /// 创建新的记忆管理器
    pub fn new(short_term_capacity: usize) -> Self {
        Self {
            short_term: ShortTermMemory::new(short_term_capacity),
            long_term: LongTermMemory::new(),
        }
    }

    /// 获取短期记忆
    pub fn short_term(&self) -> &ShortTermMemory {
        &self.short_term
    }

    /// 获取长期记忆
    pub fn long_term(&self) -> &LongTermMemory {
        &self.long_term
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_short_term_memory() {
        let memory = ShortTermMemory::new(10);
        let entry = MemoryEntry {
            id: "test1".to_string(),
            content: "Test content".to_string(),
            timestamp: 1234567890,
            tags: vec!["test".to_string()],
        };
        
        assert!(memory.add(entry).await.is_ok());
        let retrieved = memory.get("test1").await;
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_long_term_memory() {
        let memory = LongTermMemory::new();
        let entry = MemoryEntry {
            id: "test2".to_string(),
            content: "Long term content".to_string(),
            timestamp: 1234567890,
            tags: vec!["important".to_string()],
        };
        
        assert!(memory.store(entry).await.is_ok());
        let retrieved = memory.retrieve("test2").await;
        assert!(retrieved.is_some());
    }
}
