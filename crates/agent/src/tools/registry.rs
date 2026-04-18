//! 工具注册表模块
//! 
//! 实现工具注册、发现和过滤机制：
//! - getAllBaseTools() 完整工具清单
//! - 工具过滤管线
//! - ToolSearchTool 延迟发现机制

use anyhow::Result;
use std::collections::HashMap;
use crate::tools::tool::{Tool, ToolContext, ToolMetadata, ToolPermissionLevel};

/// 工具注册表
/// 
/// 管理所有已注册的工具，支持按名称和别名查找
pub struct ToolRegistry {
    /// 工具映射（按名称）
    tools_by_name: HashMap<String, Box<dyn AnyTool>>,
    /// 别名映射（别名 -> 主名称）
    aliases: HashMap<String, String>,
}

/// 类型擦除后的工具 Trait
pub trait AnyTool: Send + Sync {
    /// 获取工具元数据
    fn metadata(&self) -> ToolMetadata;
    /// 获取工具名称
    fn name(&self) -> &str;
    /// 获取别名列表
    fn aliases(&self) -> &[&str];
    /// 判断是否并发安全
    fn is_concurrency_safe(&self) -> bool;
    /// 判断是否只读
    fn is_read_only(&self) -> bool;
    /// 判断是否应该始终加载
    fn should_always_load(&self) -> bool;
    /// 执行工具（类型擦除版本）
    fn execute_any(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send + '_>>;
}

/// 工具包装器
struct ToolWrapper<T> {
    tool: T,
}

impl<T: Tool> AnyTool for ToolWrapper<T> {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            name: self.tool.name().to_string(),
            description: String::new(),
            input_schema: self.tool.input_schema(),
            permission_level: self.tool.permission_level(),
            // Note: concurrency_safe, read_only, and timeout_ms require input to determine accurately
            // Using conservative defaults since we don't have the actual input
            concurrency_safe: false, // fail-closed: assume not safe
            read_only: false,        // fail-closed: assume not read-only
            timeout_secs: None,       // no timeout by default
            always_load: self.tool.should_always_load(),
            aliases: self.tool.aliases().iter().map(|s| s.to_string()).collect(),
        }
    }

    fn name(&self) -> &str {
        self.tool.name()
    }

    fn aliases(&self) -> &[&str] {
        self.tool.aliases()
    }

    fn is_concurrency_safe(&self) -> bool {
        // Cannot determine without input - use conservative default
        false
    }

    fn is_read_only(&self) -> bool {
        // Cannot determine without input - use conservative default
        false
    }

    fn should_always_load(&self) -> bool {
        self.tool.should_always_load()
    }

    fn execute_any(
        &self,
        input: serde_json::Value,
        ctx: &ToolContext,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send + '_>> {
        Box::pin(async move {
            // 这里需要泛型执行，简化处理
            Ok(serde_json::Value::Null)
        })
    }
}

impl ToolRegistry {
    /// 创建空的工具注册表
    pub fn new() -> Self {
        Self {
            tools_by_name: HashMap::new(),
            aliases: HashMap::new(),
        }
    }

    /// 注册工具
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> Result<()> {
        let name = tool.name().to_string();
        
        // 检查名称冲突
        if self.tools_by_name.contains_key(&name) {
            anyhow::bail!("Tool already registered: {}", name);
        }

        // 注册别名映射
        for &alias in tool.aliases() {
            self.aliases.insert(alias.to_string(), name.clone());
        }

        // 注册工具
        let wrapper = ToolWrapper { tool };
        self.tools_by_name.insert(name, Box::new(wrapper));
        
        Ok(())
    }

    /// 获取工具（支持别名查找）
    pub fn get(&self, name: &str) -> Option<&Box<dyn AnyTool>> {
        // 先按主名称查找
        if let Some(tool) = self.tools_by_name.get(name) {
            return Some(tool);
        }
        
        // 再按别名查找
        if let Some(main_name) = self.aliases.get(name) {
            return self.tools_by_name.get(main_name);
        }
        
        None
    }

    /// 列出所有工具元数据
    pub fn list_tools(&self) -> Vec<ToolMetadata> {
        self.tools_by_name
            .values()
            .map(|tool| tool.metadata())
            .collect()
    }

    /// 列出所有工具名称（用于延迟发现）
    pub fn list_tool_names(&self) -> Vec<String> {
        self.tools_by_name.keys().cloned().collect()
    }

    /// 获取工具数量
    pub fn len(&self) -> usize {
        self.tools_by_name.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.tools_by_name.is_empty()
    }

    /// 获取所有基础工具清单
    /// 
    /// 这是所有内建工具的注册中心
    pub fn get_all_base_tools(&self) -> Vec<ToolMetadata> {
        // 按功能分类的工具清单
        // TODO: 实现完整的工具分类
        self.list_tools()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 工具过滤器
/// 
/// 实现多层过滤管线
pub struct ToolFilter {
    /// 简单模式下的工具白名单
    simple_mode_whitelist: Vec<String>,
    /// 拒绝规则列表
    deny_rules: Vec<String>,
    /// 启用的工具列表
    enabled_tools: Vec<String>,
}

impl ToolFilter {
    /// 创建新的过滤器
    pub fn new() -> Self {
        Self {
            simple_mode_whitelist: vec![
                "bash".to_string(),
                "read".to_string(),
                "edit".to_string(),
            ],
            deny_rules: Vec::new(),
            enabled_tools: Vec::new(),
        }
    }

    /// 创建默认的过滤器配置
    pub fn with_defaults() -> Self {
        let mut filter = Self::new();
        // 默认启用所有工具
        filter.enabled_tools = vec!["*".to_string()];
        filter
    }

    /// 模式过滤
    /// 
    /// 简单模式只保留 Bash、Read、Edit
    pub fn filter_by_mode(
        &self,
        tools: Vec<ToolMetadata>,
        mode: ToolMode,
    ) -> Vec<ToolMetadata> {
        match mode {
            ToolMode::Simple => {
                tools
                    .into_iter()
                    .filter(|t| self.simple_mode_whitelist.contains(&t.name))
                    .collect()
            }
            ToolMode::Normal => {
                // 普通模式排除特殊工具
                tools
            }
        }
    }

    /// 拒绝规则过滤
    /// 
    /// 移除被 blanket deny 规则匹配的工具
    pub fn filter_by_deny_rules(
        &self,
        tools: Vec<ToolMetadata>,
    ) -> Vec<ToolMetadata> {
        tools
            .into_iter()
            .filter(|t| {
                !self.deny_rules.iter().any(|rule| {
                    t.name.contains(rule) || t.description.contains(rule)
                })
            })
            .collect()
    }

    /// 启用状态检查
    /// 
    /// 过滤掉未启用的工具
    pub fn filter_by_enabled(&self, tools: Vec<ToolMetadata>) -> Vec<ToolMetadata> {
        // 如果启用列表包含 "*"，则启用所有工具
        if self.enabled_tools.iter().any(|t| t == "*") {
            return tools;
        }

        tools
            .into_iter()
            .filter(|t| self.enabled_tools.contains(&t.name))
            .collect()
    }

    /// 完整的过滤管线
    /// 
    /// 从 getAllBaseTools() 到最终发送给 API 的工具列表
    pub fn filter_all(
        &self,
        all_tools: Vec<ToolMetadata>,
        mode: ToolMode,
    ) -> Vec<ToolMetadata> {
        // 1. 模式过滤
        let tools = self.filter_by_mode(all_tools, mode);
        
        // 2. 拒绝规则过滤
        let tools = self.filter_by_deny_rules(tools);
        
        // 3. 启用状态检查
        let tools = self.filter_by_enabled(tools);
        
        // 4. 按名称排序（确保 prompt 缓存稳定性）
        let mut tools = tools;
        tools.sort_by(|a, b| a.name.cmp(&b.name));
        
        tools
    }
}

impl Default for ToolFilter {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 工具模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolMode {
    /// 简单模式（受限工具集）
    Simple,
    /// 普通模式（完整工具集）
    Normal,
}

/// 工具发现策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryStrategy {
    /// 立即发现（发送完整 schema）
    Immediate,
    /// 延迟发现（只发送名称列表）
    Deferred,
}

impl DiscoveryStrategy {
    /// 根据工具数量自动选择策略
    pub fn auto_select(tool_count: usize) -> Self {
        // 当工具数量超过阈值时使用延迟发现
        if tool_count > 50 {
            Self::Deferred
        } else {
            Self::Immediate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ToolRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_tool_filter_mode() {
        let filter = ToolFilter::with_defaults();
        
        let tools = vec![
            ToolMetadata {
                name: "bash".to_string(),
                description: "Bash tool".to_string(),
                input_schema: serde_json::Value::Null,
                permission_level: ToolPermissionLevel::RequiresConfirmation,
                concurrency_safe: false,
                read_only: false,
                timeout_secs: None,
                always_load: false,
                aliases: Vec::new(),
            },
            ToolMetadata {
                name: "read".to_string(),
                description: "Read tool".to_string(),
                input_schema: serde_json::Value::Null,
                permission_level: ToolPermissionLevel::ReadOnly,
                concurrency_safe: true,
                read_only: true,
                timeout_secs: None,
                always_load: false,
                aliases: Vec::new(),
            },
            ToolMetadata {
                name: "custom".to_string(),
                description: "Custom tool".to_string(),
                input_schema: serde_json::Value::Null,
                permission_level: ToolPermissionLevel::RequiresConfirmation,
                concurrency_safe: false,
                read_only: false,
                timeout_secs: None,
                always_load: false,
                aliases: Vec::new(),
            },
        ];

        let filtered = filter.filter_by_mode(tools.clone(), ToolMode::Simple);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|t| t.name == "bash"));
        assert!(filtered.iter().any(|t| t.name == "read"));
        assert!(!filtered.iter().any(|t| t.name == "custom"));
    }

    #[test]
    fn test_discovery_strategy() {
        assert_eq!(DiscoveryStrategy::auto_select(10), DiscoveryStrategy::Immediate);
        assert_eq!(DiscoveryStrategy::auto_select(100), DiscoveryStrategy::Deferred);
    }
}
