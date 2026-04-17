//! 功能开关系统模块
//! 
//! 实现双层功能开关：
//! - 编译时：feature() 函数，死代码消除
//! - 运行时：GrowthBook 实验框架

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 编译时功能标志
/// 
/// 这些标志在编译时确定，用于死代码消除
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompileTimeFeature {
    /// Assistant 模式（长驻会话）
    Kairos,
    /// 后台记忆提取
    ExtractMemories,
    /// 自动模式分类器
    TranscriptClassifier,
    /// 团队记忆
    TeamMem,
    /// Computer Use MCP
    ChicagoMcp,
    /// 模板与工作流
    Templates,
    /// 伴侣精灵
    Buddy,
    /// 后台守护进程
    Daemon,
    /// 桥接模式
    BridgeMode,
}

impl CompileTimeFeature {
    /// 获取功能标志名称
    pub fn name(&self) -> &'static str {
        match self {
            Self::Kairos => "KAIROS",
            Self::ExtractMemories => "EXTRACT_MEMORIES",
            Self::TranscriptClassifier => "TRANSCRIPT_CLASSIFIER",
            Self::TeamMem => "TEAMMEM",
            Self::ChicagoMcp => "CHICAGO_MCP",
            Self::Templates => "TEMPLATES",
            Self::Buddy => "BUDDY",
            Self::Daemon => "DAEMON",
            Self::BridgeMode => "BRIDGE_MODE",
        }
    }

    /// 从名称解析功能标志
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "KAIROS" => Some(Self::Kairos),
            "EXTRACT_MEMORIES" => Some(Self::ExtractMemories),
            "TRANSCRIPT_CLASSIFIER" => Some(Self::TranscriptClassifier),
            "TEAMMEM" => Some(Self::TeamMem),
            "CHICAGO_MCP" => Some(Self::ChicagoMcp),
            "TEMPLATES" => Some(Self::Templates),
            "BUDDY" => Some(Self::Buddy),
            "DAEMON" => Some(Self::Daemon),
            "BRIDGE_MODE" => Some(Self::BridgeMode),
            _ => None,
        }
    }

    /// 获取功能标志的默认启用状态
    /// 
    /// 实际启用状态由构建系统决定
    pub fn default_enabled(&self) -> bool {
        // 这些是示例默认值，实际应该由编译时配置决定
        match self {
            Self::Kairos => true,
            Self::ExtractMemories => false,
            Self::TranscriptClassifier => true,
            Self::TeamMem => false,
            Self::ChicagoMcp => false,
            Self::Templates => false,
            Self::Buddy => false,
            Self::Daemon => false,
            Self::BridgeMode => false,
        }
    }
}

/// 运行时功能标志（GrowthBook 实验）
/// 
/// 这些标志使用随机命名的动物名称，避免语义偏见
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RuntimeFeature {
    /// 功能标志名称（如 "tengu_passport_quail"）
    pub name: String,
    /// 默认值
    pub default_value: bool,
}

impl RuntimeFeature {
    /// 创建新的运行时功能标志
    pub fn new(name: impl Into<String>, default_value: bool) -> Self {
        Self {
            name: name.into(),
            default_value,
        }
    }
}

/// 功能标志管理器
/// 
/// 统一管理编译时和运行时功能标志
pub struct FeatureManager {
    /// 编译时功能标志状态
    compile_time_features: Arc<RwLock<HashMap<CompileTimeFeature, bool>>>,
    /// 运行时功能标志状态（来自 GrowthBook）
    runtime_features: Arc<RwLock<HashMap<String, bool>>>,
    /// 是否已缓存运行时特征（可能过期）
    runtime_features_cached: Arc<RwLock<bool>>,
}

impl FeatureManager {
    /// 创建新的功能标志管理器
    pub fn new() -> Self {
        let mut compile_time = HashMap::new();
        
        // 初始化编译时功能标志
        for feature in [
            CompileTimeFeature::Kairos,
            CompileTimeFeature::ExtractMemories,
            CompileTimeFeature::TranscriptClassifier,
            CompileTimeFeature::TeamMem,
            CompileTimeFeature::ChicagoMcp,
            CompileTimeFeature::Templates,
            CompileTimeFeature::Buddy,
            CompileTimeFeature::Daemon,
            CompileTimeFeature::BridgeMode,
        ] {
            compile_time.insert(feature, feature.default_enabled());
        }

        Self {
            compile_time_features: Arc::new(RwLock::new(compile_time)),
            runtime_features: Arc::new(RwLock::new(HashMap::new())),
            runtime_features_cached: Arc::new(RwLock::new(false)),
        }
    }

    /// 检查编译时功能标志
    /// 
    /// 当返回 false 时，bundler 应该完全移除对应的代码分支
    pub fn feature(&self, feature: CompileTimeFeature) -> bool {
        self.compile_time_features
            .read()
            .unwrap()
            .get(&feature)
            .copied()
            .unwrap_or(false)
    }

    /// 获取运行时功能标志值
    /// 
    /// 返回值来自缓存，可能在跨进程场景下已过时
    /// 这是为了在启动关键路径上避免异步等待
    pub fn get_feature_value_cached(&self, name: &str) -> Option<bool> {
        let cached = *self.runtime_features_cached.read().unwrap();
        if !cached {
            return None;
        }

        self.runtime_features
            .read()
            .unwrap()
            .get(name)
            .copied()
    }

    /// 获取运行时功能标志值（带默认值）
    pub fn get_feature_value_with_default(&self, name: &str, default: bool) -> bool {
        self.get_feature_value_cached(name).unwrap_or(default)
    }

    /// 更新运行时功能标志
    /// 
    /// 通常从 GrowthBook API 同步
    pub fn update_runtime_features(&self, features: HashMap<String, bool>) {
        let mut runtime = self.runtime_features.write().unwrap();
        *runtime = features;
        
        let mut cached = self.runtime_features_cached.write().unwrap();
        *cached = true;
    }

    /// 清除运行时功能标志缓存
    pub fn invalidate_runtime_cache(&self) {
        let mut cached = self.runtime_features_cached.write().unwrap();
        *cached = false;
    }

    /// 强制设置编译时功能标志
    /// 
    /// 仅用于测试，实际应该由构建系统决定
    #[cfg(test)]
    pub fn set_compile_time_feature(&self, feature: CompileTimeFeature, enabled: bool) {
        let mut features = self.compile_time_features.write().unwrap();
        features.insert(feature, enabled);
    }
}

impl Default for FeatureManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 简化的功能检查宏
/// 
/// 用法：`feature!("KAIROS")`
/// 
/// 在编译时展开为常量
#[macro_export]
macro_rules! feature {
    ($name:literal) => {{
        use crate::settings::feature_flags::CompileTimeFeature;
        
        if let Some(ft) = CompileTimeFeature::from_name($name) {
            ft.default_enabled()
        } else {
            false
        }
    }};
}

/// 运行时特征检查宏
/// 
/// 用法：`runtime_feature!("tengu_passport_quail", false)`
#[macro_export]
macro_rules! runtime_feature {
    ($name:literal, $default:expr) => {{
        use crate::settings::feature_flags::FeatureManager;
        
        // 获取全局管理器实例
        FeatureManager::default().get_feature_value_with_default($name, $default)
    }};
}

/// GrowthBook 风格的随机命名功能标志
/// 
/// 使用动物名称组合避免语义偏见
pub mod growthbook_features {
    /// 记忆系统相关的随机功能标志示例
    pub const MEMORY_FEATURES: &[&str] = &[
        "tengu_passport_quail",
        "tengu_coral_fern",
        "tengu_moth_copse",
    ];

    /// 权限系统相关的随机功能标志示例
    pub const PERMISSION_FEATURES: &[&str] = &[
        "tengo_yucca_pine",
        "tengo_sage_grouse",
    ];

    /// UI 相关的随机功能标志示例
    pub const UI_FEATURES: &[&str] = &[
        "tengu_aspen_grove",
        "tengu_willow_creek",
    ];
}

/// 功能标志决策树
/// 
/// 帮助开发者选择使用编译时还是运行时开关
pub mod decision_tree {
    /// 决定使用哪种功能开关类型
    pub enum FeatureSwitchType {
        /// 编译时开关 - feature()
        CompileTime,
        /// 运行时开关 - GrowthBook
        Runtime,
        /// A/B 测试 - GrowthBook + 随机分组
        ABTest,
        /// 不需要开关 - 直接硬编码
        Hardcoded,
    }

    /// 判断是否需要运行时动态控制
    pub fn needs_runtime_control(feature_description: &str) -> bool {
        // 以下场景需要运行时控制：
        // 1. 需要在发布后快速迭代
        // 2. 需要根据用户行为动态调整
        // 3. 需要 A/B 测试
        
        // 示例：
        // - "自动模式分类器的置信度阈值" → 需要运行时控制
        // - "是否启用新的 UI 布局" → 需要 A/B 测试
        // - "是否编译 Assistant 模式" → 编译时控制
        
        true // 简化实现
    }

    /// 判断是否需要 A/B 测试
    pub fn needs_ab_test(feature_description: &str) -> bool {
        // 以下场景需要 A/B 测试：
        // 1. 需要对比两种实现的效果
        // 2. 需要随机分组减少偏差
        // 3. 需要统计显著性验证
        
        false // 简化实现
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_time_feature_names() {
        assert_eq!(CompileTimeFeature::Kairos.name(), "KAIROS");
        assert_eq!(CompileTimeFeature::TranscriptClassifier.name(), "TRANSCRIPT_CLASSIFIER");
    }

    #[test]
    fn test_feature_from_name() {
        assert_eq!(
            CompileTimeFeature::from_name("KAIROS"),
            Some(CompileTimeFeature::Kairos)
        );
        assert_eq!(
            CompileTimeFeature::from_name("UNKNOWN"),
            None
        );
    }

    #[test]
    fn test_feature_manager() {
        let manager = FeatureManager::new();
        
        // 测试编译时功能
        assert!(manager.feature(CompileTimeFeature::Kairos));
        assert!(!manager.feature(CompileTimeFeature::ExtractMemories));
        
        // 测试运行时功能
        assert_eq!(manager.get_feature_value_cached("test"), None);
        
        // 更新运行时功能
        let mut features = HashMap::new();
        features.insert("test".to_string(), true);
        manager.update_runtime_features(features);
        
        assert_eq!(manager.get_feature_value_cached("test"), Some(true));
        
        // 测试带默认值
        assert_eq!(manager.get_feature_value_with_default("nonexistent", true), true);
        assert_eq!(manager.get_feature_value_with_default("nonexistent", false), false);
    }

    #[test]
    fn test_runtime_feature_invalidation() {
        let manager = FeatureManager::new();
        
        // 初始缓存应该是 false
        assert!(!*manager.runtime_features_cached.read().unwrap());
        
        // 更新后应该是 true
        manager.update_runtime_features(HashMap::new());
        assert!(*manager.runtime_features_cached.read().unwrap());
        
        // 清除后应该是 false
        manager.invalidate_runtime_cache();
        assert!(!*manager.runtime_features_cached.read().unwrap());
    }

    #[test]
    fn test_growthbook_feature_names() {
        // 验证随机命名格式
        for name in growthbook_features::MEMORY_FEATURES {
            assert!(name.starts_with("tengu_"));
            assert!(name.contains('_'));
        }
    }
}
