//! 有界 UUID 集合
//!
//! 用于去重，自动保持最多 N 个最近的 UUID

use std::collections::HashSet;
use std::collections::VecDeque;

/// 有界 UUID 集合
pub struct BoundedUuidSet {
    /// 存储 UUID 的集合
    set: HashSet<String>,
    /// FIFO 队列，用于追踪插入顺序
    queue: VecDeque<String>,
    /// 最大容量
    max_size: usize,
}

impl BoundedUuidSet {
    /// 创建新的有界集合
    pub fn new(max_size: usize) -> Self {
        Self {
            set: HashSet::with_capacity(max_size),
            queue: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// 添加 UUID（如果已满，移除最旧的）
    pub fn insert(&mut self, uuid: String) {
        // 如果已存在，不需要重复添加
        if self.set.contains(&uuid) {
            return;
        }

        // 如果已满，移除最旧的
        if self.set.len() >= self.max_size {
            if let Some(oldest) = self.queue.pop_front() {
                self.set.remove(&oldest);
            }
        }

        // 添加新的
        self.set.insert(uuid.clone());
        self.queue.push_back(uuid);
    }

    /// 检查 UUID 是否存在
    pub fn contains(&self, uuid: &str) -> bool {
        self.set.contains(uuid)
    }

    /// 移除 UUID
    pub fn remove(&mut self, uuid: &str) -> bool {
        if let Some(pos) = self.queue.iter().position(|u| u == uuid) {
            self.queue.remove(pos);
        }
        self.set.remove(uuid)
    }

    /// 清空集合
    pub fn clear(&mut self) {
        self.set.clear();
        self.queue.clear();
    }

    /// 获取当前大小
    pub fn len(&self) -> usize {
        self.set.len()
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    /// 获取容量上限
    pub fn capacity(&self) -> usize {
        self.max_size
    }
}

impl Default for BoundedUuidSet {
    fn default() -> Self {
        // 默认保持 10000 个 UUID
        Self::new(10000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_insert() {
        let mut set = BoundedUuidSet::new(3);

        set.insert("a".to_string());
        set.insert("b".to_string());
        set.insert("c".to_string());

        assert!(set.contains("a"));
        assert!(set.contains("b"));
        assert!(set.contains("c"));
        assert_eq!(set.len(), 3);

        // 添加第 4 个，应该移除 "a"
        set.insert("d".to_string());

        assert!(!set.contains("a"));
        assert!(set.contains("b"));
        assert!(set.contains("c"));
        assert!(set.contains("d"));
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn test_duplicate_insert() {
        let mut set = BoundedUuidSet::new(3);

        set.insert("a".to_string());
        set.insert("a".to_string()); // 重复插入

        assert_eq!(set.len(), 1);
        assert!(set.contains("a"));
    }

    #[test]
    fn test_remove() {
        let mut set = BoundedUuidSet::new(3);

        set.insert("a".to_string());
        set.insert("b".to_string());

        assert!(set.remove("a"));
        assert!(!set.contains("a"));
        assert!(set.contains("b"));
        assert_eq!(set.len(), 1);
    }
}
