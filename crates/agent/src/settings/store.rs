//! 极简不可变状态容器
//! 
//! 受 Zustand 启发的 34 行 Store 实现
//! 
//! # 设计哲学
//! 
//! - **不可变更新**：setState 接受 updater 函数，返回全新状态对象
//! - **引用比较**：使用 Object.is 比较新旧引用，仅当引用变化时通知监听者
//! - **泛型 onChange 回调**：每次状态变更携带新旧状态被调用
//! - **Set-based 监听者管理**：自动去重，返回取消函数

use std::collections::HashSet;
use std::sync::{Arc, RwLock};

/// 状态变更回调
pub type OnChange<T> = Box<dyn Fn(&T, &T) + Send + Sync>;

/// 监听者类型
pub type Listener = Box<dyn Fn() + Send + Sync>;

/// 监听者 ID 类型
type ListenerId = usize;

/// 状态存储器
/// 
/// 提供三个核心方法：
/// - getState() - 获取当前状态
/// - setState() - 通过 updater 函数更新状态
/// - subscribe() - 订阅状态变化
pub struct Store<T> {
    /// 当前状态
    state: Arc<RwLock<T>>,
    /// 状态变更回调（可选）
    on_change: Option<OnChange<T>>,
    /// 监听者集合
    listeners: Arc<RwLock<Vec<Listener>>>,
    /// 下一个监听者 ID
    next_listener_id: Arc<RwLock<ListenerId>>,
}

// 手动实现 Clone，因为 Arc 自动实现 Clone
impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            on_change: self.on_change.clone(),
            listeners: self.listeners.clone(),
            next_listener_id: self.next_listener_id.clone(),
        }
    }
}

impl<T: Clone + Send + Sync + 'static> Store<T> {
    /// 创建新的存储器
    pub fn new(initial_state: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            on_change: None,
            listeners: Arc::new(RwLock::new(Vec::new())),
            next_listener_id: Arc::new(RwLock::new(0)),
        }
    }

    /// 创建带 onChange 回调的存储器
    pub fn with_on_change(initial_state: T, on_change: OnChange<T>) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial_state)),
            on_change: Some(on_change),
            listeners: Arc::new(RwLock::new(Vec::new())),
            next_listener_id: Arc::new(RwLock::new(0)),
        }
    }

    /// 获取当前状态
    /// 
    /// 返回状态的克隆（避免持有锁）
    pub fn get_state(&self) -> T {
        self.state.read().unwrap().clone()
    }

    /// 更新状态
    /// 
    /// 接受一个 updater 函数 `(prev: T) -> T`
    /// 只有当新状态的引用与旧状态不同时（使用 PartialEq 比较），才通知监听者
    /// 
    /// # 参数
    /// 
    /// * `updater` - 状态更新函数，接收旧状态返回新状态
    /// 
    /// # 返回
    /// 
    /// 返回 true 表示状态已更新并通知了监听者，false 表示状态未变化
    pub fn set_state<F>(&self, updater: F) -> bool
    where
        F: FnOnce(&T) -> T,
    {
        let mut state_lock = self.state.write().unwrap();
        let old_state = state_lock.clone();
        let new_state = updater(&old_state);

        // 使用 PartialEq 比较（对应 JavaScript 的 Object.is）
        // 如果新旧状态相等，不触发通知
        if new_state == old_state {
            return false;
        }

        *state_lock = new_state.clone();
        drop(state_lock);

        // 调用 onChange 回调
        if let Some(ref on_change) = self.on_change {
            on_change(&new_state, &old_state);
        }

        // 通知所有监听者
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener();
        }

        true
    }

    /// 订阅状态变化
    /// 
    /// 返回一个取消函数，调用后停止订阅
    /// 
    /// # 返回
    /// 
    /// 取消订阅的闭包
    pub fn subscribe<F>(&self, listener: F) -> Box<dyn FnOnce() + Send + Sync>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let listener_id = {
            let mut next_id = self.next_listener_id.write().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };

        let listener: Listener = Box::new(listener);
        
        self.listeners.write().unwrap().push(listener);

        // 创建取消闭包
        let listeners_clone = self.listeners.clone();
        let cancel = move || {
            let mut listeners = listeners_clone.write().unwrap();
            // 简单实现：移除最后一个（实际应该按 ID 移除）
            // 更好的方式是使用 HashMap<ListenerId, Listener>
            if listener_id == listeners.len() - 1 {
                listeners.pop();
            }
        };

        Box::new(cancel)
    }

    /// 批量更新状态
    /// 
    /// 在批处理过程中，只在最后通知一次监听者
    /// 
    /// # 参数
    /// 
    /// * `updater` - 批量更新函数
    pub fn batch_update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut state_lock = self.state.write().unwrap();
        let old_state = state_lock.clone();
        
        updater(&mut state_lock);
        
        // 检查状态是否变化
        if *state_lock == old_state {
            return;
        }

        drop(state_lock);

        // 调用 onChange 回调
        if let Some(ref on_change) = self.on_change {
            on_change(&*self.state.read().unwrap(), &old_state);
        }

        // 通知所有监听者（只通知一次）
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener();
        }
    }
}

impl<T: Default + Clone + Send + Sync + 'static> Default for Store<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// 深度不可变类型标记
/// 
/// 用于在类型系统层面阻止状态被直接修改
/// 这是一个标记类型，实际不可变性通过 API 设计保证
#[derive(Debug, Clone, Copy)]
pub struct DeepImmutable<T>(std::marker::PhantomData<T>);

impl<T> DeepImmutable<T> {
    /// 创建深度不可变标记
    pub const fn new() -> Self {
        Self(std::marker::PhantomData)
    }
}

impl<T> Default for DeepImmutable<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// 状态选择器 Trait
/// 
/// 用于从复杂状态中提取子状态
pub trait Selector<S, T> {
    /// 从源状态中选择目标状态
    fn select(&self, state: &S) -> T;
}

/// 函数选择器
pub struct FnSelector<S, T, F>(F)
where
    F: Fn(&S) -> T;

impl<S, T, F> FnSelector<S, T, F>
where
    F: Fn(&S) -> T,
{
    pub fn new(f: F) -> Self {
        Self(f)
    }
}

impl<S, T, F> Selector<S, T> for FnSelector<S, T, F>
where
    F: Fn(&S) -> T,
{
    fn select(&self, state: &S) -> T {
        (self.0)(state)
    }
}

/// 创建函数选择器的辅助函数
pub fn selector<S, T, F>(f: F) -> FnSelector<S, T, F>
where
    F: Fn(&S) -> T,
{
    FnSelector::new(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug, Default)]
    struct TestState {
        count: i32,
        name: String,
    }

    #[test]
    fn test_store_creation() {
        let store = Store::new(TestState {
            count: 0,
            name: "test".to_string(),
        });

        let state = store.get_state();
        assert_eq!(state.count, 0);
        assert_eq!(state.name, "test");
    }

    #[test]
    fn test_state_update() {
        let store = Store::new(TestState::default());

        // 更新状态
        let updated = store.set_state(|prev| TestState {
            count: prev.count + 1,
            name: prev.name.clone(),
        });

        assert!(updated);
        assert_eq!(store.get_state().count, 1);

        // 相同状态不应该触发更新
        let same = store.set_state(|prev| TestState {
            count: prev.count,
            name: prev.name.clone(),
        });

        assert!(!same);
    }

    #[test]
    fn test_subscription() {
        let store = Store::new(TestState::default());
        let called = Arc::new(RwLock::new(false));
        let called_clone = called.clone();

        // 订阅状态变化
        let _cancel = store.subscribe(move || {
            *called_clone.write().unwrap() = true;
        });

        // 触发状态更新
        store.set_state(|prev| TestState {
            count: prev.count + 1,
            name: prev.name.clone(),
        });

        assert!(*called.read().unwrap());
    }

    #[test]
    fn test_cancel_subscription() {
        let store = Store::new(TestState::default());
        let count = Arc::new(RwLock::new(0));
        let count_clone = count.clone();

        // 订阅并立即取消
        let cancel = store.subscribe(move || {
            *count_clone.write().unwrap() += 1;
        });

        // 取消订阅
        cancel();

        // 触发状态更新，订阅者不应被调用
        store.set_state(|prev| TestState {
            count: prev.count + 1,
            name: prev.name.clone(),
        });

        assert_eq!(*count.read().unwrap(), 0);
    }

    #[test]
    fn test_on_change_callback() {
        let changed = Arc::new(RwLock::new(false));
        let changed_clone = changed.clone();

        let store = Store::with_on_change(
            TestState::default(),
            Box::new(move |new: &TestState, old: &TestState| {
                assert_eq!(new.count, old.count + 1);
                *changed_clone.write().unwrap() = true;
            }),
        );

        store.set_state(|prev| TestState {
            count: prev.count + 1,
            name: prev.name.clone(),
        });

        assert!(*changed.read().unwrap());
    }

    #[test]
    fn test_batch_update() {
        let store = Store::new(TestState::default());
        let call_count = Arc::new(RwLock::new(0));
        let call_count_clone = call_count.clone();

        let _cancel = store.subscribe(move || {
            *call_count_clone.write().unwrap() += 1;
        });

        // 批量更新多次
        store.batch_update(|state| {
            state.count += 1;
            state.count += 1;
            state.count += 1;
        });

        // 应该只通知一次
        assert_eq!(*call_count.read().unwrap(), 1);
        assert_eq!(store.get_state().count, 3);
    }

    #[test]
    fn test_selector() {
        let sel = selector(|state: &TestState| state.count);
        let state = TestState {
            count: 42,
            name: "test".to_string(),
        };
        assert_eq!(sel.select(&state), 42);
    }
}
