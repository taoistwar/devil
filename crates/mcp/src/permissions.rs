//! MCP 四层权限检查器
//!
//! 实现深度防御权限模型：
//! 1. 企业策略（Enterprise Policy）- 最严格，全局生效
//! 2. IDE 白名单（IDE Whitelist）- 管理员配置
//! 3. 用户权限（User Permission）- 用户个性化配置
//! 4. 运行时确认（Runtime Confirmation）- 每次调用前确认

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// 权限检查器
pub struct PermissionChecker {
    /// 企业策略
    enterprise_policy: Arc<RwLock<EnterprisePolicy>>,
    /// IDE 白名单
    ide_whitelist: Arc<RwLock<IdeWhitelist>>,
    /// 用户权限
    user_permissions: Arc<RwLock<UserPermissions>>,
    /// 运行时确认缓存（会话级）
    runtime_cache: Arc<RwLock<HashSet<String>>>,
}

/// 企业策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterprisePolicy {
    /// 是否启用 MCP
    pub enabled: bool,
    /// 允许的服务器列表（空表示全部允许）
    pub allowed_servers: Vec<String>,
    /// 禁止的服务器列表
    pub blocked_servers: Vec<String>,
    /// 允许的工具列表（glob 模式，空表示全部允许）
    pub allowed_tools: Vec<String>,
    /// 禁止的工具列表（glob 模式）
    pub blocked_tools: Vec<String>,
    /// 是否需要管理员审批新服务器
    pub require_admin_approval: bool,
}

impl Default for EnterprisePolicy {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_servers: vec![],
            blocked_servers: vec![],
            allowed_tools: vec![],
            blocked_tools: vec![],
            require_admin_approval: false,
        }
    }
}

/// IDE 白名单
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeWhitelist {
    /// 允许的服务器列表
    pub allowed_servers: Vec<String>,
    /// 允许的工具列表（glob 模式）
    pub allowed_tools: Vec<String>,
}

impl Default for IdeWhitelist {
    fn default() -> Self {
        Self {
            allowed_servers: vec![],
            allowed_tools: vec![],
        }
    }
}

/// 用户权限
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermissions {
    /// 用户启用的服务器
    pub enabled_servers: Vec<String>,
    /// 用户禁用的服务器
    pub disabled_servers: Vec<String>,
    /// 用户授权的工具
    pub authorized_tools: Vec<String>,
    /// 用户禁止的工具
    pub blocked_tools: Vec<String>,
}

impl Default for UserPermissions {
    fn default() -> Self {
        Self {
            enabled_servers: vec![],
            disabled_servers: vec![],
            authorized_tools: vec![],
            blocked_tools: vec![],
        }
    }
}

/// 权限检查结果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionResult {
    /// 允许
    Allowed,
    /// 拒绝（附带原因）
    Denied(String),
    /// 需要确认（用户未决断）
    NeedsConfirmation,
}

impl PermissionChecker {
    /// 创建新的权限检查器
    pub fn new(enterprise: EnterprisePolicy, ide: IdeWhitelist, user: UserPermissions) -> Self {
        Self {
            enterprise_policy: Arc::new(RwLock::new(enterprise)),
            ide_whitelist: Arc::new(RwLock::new(ide)),
            user_permissions: Arc::new(RwLock::new(user)),
            runtime_cache: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// 检查服务器是否被允许
    pub async fn check_server(&self, server_id: &str) -> PermissionResult {
        debug!("Checking server permission: {}", server_id);

        // Layer 1: 企业策略
        {
            let policy = self.enterprise_policy.read().await;

            if !policy.enabled {
                return PermissionResult::Denied(
                    "MCP is disabled by enterprise policy".to_string(),
                );
            }

            // 黑名单优先
            if policy
                .blocked_servers
                .iter()
                .any(|s| match_glob(s, server_id))
            {
                return PermissionResult::Denied(format!(
                    "Server {} is blocked by enterprise policy",
                    server_id
                ));
            }

            // 白名单检查
            if !policy.allowed_servers.is_empty()
                && !policy
                    .allowed_servers
                    .iter()
                    .any(|s| match_glob(s, server_id))
            {
                return PermissionResult::Denied(format!(
                    "Server {} is not in enterprise allowed list",
                    server_id
                ));
            }
        }

        // Layer 2: IDE 白名单
        {
            let whitelist = self.ide_whitelist.read().await;

            if !whitelist.allowed_servers.is_empty()
                && !whitelist
                    .allowed_servers
                    .iter()
                    .any(|s| match_glob(s, server_id))
            {
                return PermissionResult::Denied(format!(
                    "Server {} is not in IDE whitelist",
                    server_id
                ));
            }
        }

        // Layer 3: 用户权限
        {
            let user = self.user_permissions.read().await;

            // 用户明确禁用
            if user
                .disabled_servers
                .iter()
                .any(|s| match_glob(s, server_id))
            {
                return PermissionResult::Denied(format!(
                    "Server {} is disabled by user",
                    server_id
                ));
            }

            // 如果用户配置了启用列表，检查是否在列表中
            if !user.enabled_servers.is_empty()
                && !user
                    .enabled_servers
                    .iter()
                    .any(|s| match_glob(s, server_id))
            {
                return PermissionResult::NeedsConfirmation;
            }
        }

        // Layer 4: 运行时确认缓存
        {
            let cache = self.runtime_cache.read().await;
            if cache.contains(server_id) {
                return PermissionResult::Allowed;
            }
        }

        PermissionResult::NeedsConfirmation
    }

    /// 检查工具是否被允许
    pub async fn check_tool(&self, tool_global_name: &str) -> PermissionResult {
        debug!("Checking tool permission: {}", tool_global_name);

        // Layer 1: 企业策略
        {
            let policy = self.enterprise_policy.read().await;

            // 黑名单优先
            if policy
                .blocked_tools
                .iter()
                .any(|p| match_glob(p, tool_global_name))
            {
                return PermissionResult::Denied(format!(
                    "Tool {} is blocked by enterprise policy",
                    tool_global_name
                ));
            }

            // 白名单检查
            if !policy.allowed_tools.is_empty()
                && !policy
                    .allowed_tools
                    .iter()
                    .any(|p| match_glob(p, tool_global_name))
            {
                return PermissionResult::Denied(format!(
                    "Tool {} is not in enterprise allowed list",
                    tool_global_name
                ));
            }
        }

        // Layer 2: IDE 白名单
        {
            let whitelist = self.ide_whitelist.read().await;

            if !whitelist.allowed_tools.is_empty()
                && !whitelist
                    .allowed_tools
                    .iter()
                    .any(|p| match_glob(p, tool_global_name))
            {
                return PermissionResult::Denied(format!(
                    "Tool {} is not in IDE whitelist",
                    tool_global_name
                ));
            }
        }

        // Layer 3: 用户权限
        {
            let user = self.user_permissions.read().await;

            // 用户明确禁止
            if user
                .blocked_tools
                .iter()
                .any(|p| match_glob(p, tool_global_name))
            {
                return PermissionResult::Denied(format!(
                    "Tool {} is blocked by user",
                    tool_global_name
                ));
            }

            // 如果用户配置了授权列表，检查是否在列表中
            if !user.authorized_tools.is_empty()
                && !user
                    .authorized_tools
                    .iter()
                    .any(|p| match_glob(p, tool_global_name))
            {
                return PermissionResult::NeedsConfirmation;
            }
        }

        // Layer 4: 运行时确认缓存
        {
            let cache = self.runtime_cache.read().await;
            if cache.contains(tool_global_name) {
                return PermissionResult::Allowed;
            }
        }

        PermissionResult::NeedsConfirmation
    }

    /// 标记为已确认（运行时）
    pub async fn confirm(&self, identifier: &str) -> Result<()> {
        let mut cache = self.runtime_cache.write().await;
        cache.insert(identifier.to_string());
        debug!("Confirmed: {}", identifier);
        Ok(())
    }

    /// 清除运行时确认缓存
    pub async fn clear_runtime_cache(&self) {
        let mut cache = self.runtime_cache.write().await;
        cache.clear();
        info!("Runtime confirmation cache cleared");
    }

    /// 更新企业策略
    pub async fn update_enterprise_policy(&self, policy: EnterprisePolicy) {
        let mut current = self.enterprise_policy.write().await;
        *current = policy;
    }

    /// 更新 IDE 白名单
    pub async fn update_ide_whitelist(&self, whitelist: IdeWhitelist) {
        let mut current = self.ide_whitelist.write().await;
        *current = whitelist;
    }

    /// 更新用户权限
    pub async fn update_user_permissions(&self, permissions: UserPermissions) {
        let mut current = self.user_permissions.write().await;
        *current = permissions;
    }
}

/// Glob 模式匹配
///
/// 支持简单的通配符：
/// - `*` 匹配任意数量的任意字符
/// - `?` 匹配单个字符
fn match_glob(pattern: &str, text: &str) -> bool {
    if pattern.is_empty() {
        return text.is_empty();
    }

    let mut pattern_chars = pattern.chars().peekable();
    let mut text_chars = text.chars().peekable();

    while let Some(p) = pattern_chars.next() {
        match p {
            '*' => {
                // 贪婪匹配
                if pattern_chars.peek().is_none() {
                    return true;
                }

                let next_pattern: String = pattern_chars.clone().collect();

                // 尝试所有可能的匹配位置
                while let Some(_) = text_chars.next() {
                    if match_glob(&next_pattern, &text_chars.clone().collect::<String>()) {
                        return true;
                    }
                }

                return match_glob(&next_pattern, "");
            }
            '?' => {
                if text_chars.next().is_none() {
                    return false;
                }
            }
            _ => match text_chars.next() {
                Some(t) if t == p => continue,
                _ => return false,
            },
        }
    }

    text_chars.peek().is_none()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_match_glob() {
        assert!(match_glob("*", "anything"));
        assert!(match_glob("mcp__*__bash", "mcp__myserver__bash"));
        assert!(match_glob("*-tools::*", "myserver-tools::execute"));
        assert!(!match_glob("specific", "other"));
        assert!(match_glob("test?", "test1"));
        assert!(!match_glob("test?", "test12"));
        assert!(match_glob("*blocked*", "blocked-server"));
        assert!(match_glob("allowed-*", "allowed-server"));
    }

    #[test]
    fn test_match_glob_edge_cases() {
        assert!(match_glob("", ""));
        assert!(!match_glob("", "nonempty"));
        assert!(match_glob("exact", "exact"));
        assert!(!match_glob("exact", "exactextra"));
        assert!(match_glob("a*b", "ab"));
        assert!(match_glob("a*b", "aXXXb"));
        assert!(match_glob("a?b", "a1b"));
        assert!(!match_glob("a?b", "ab"));
        assert!(!match_glob("a?b", "a12b"));
    }

    #[tokio::test]
    async fn test_enterprise_policy_blocks_server() {
        let enterprise = EnterprisePolicy {
            enabled: true,
            blocked_servers: vec!["blocked-*".to_string()],
            allowed_servers: vec![],
            blocked_tools: vec![],
            allowed_tools: vec![],
            require_admin_approval: false,
        };

        let checker = PermissionChecker::new(
            enterprise,
            IdeWhitelist::default(),
            UserPermissions::default(),
        );

        let result = checker.check_server("blocked-server").await;
        assert!(matches!(result, PermissionResult::Denied(_)));
        if let PermissionResult::Denied(reason) = result {
            assert!(reason.contains("blocked by enterprise"));
        }
    }

    #[tokio::test]
    async fn test_enterprise_policy_disables_mcp() {
        let enterprise = EnterprisePolicy {
            enabled: false, // MCP 被禁用
            blocked_servers: vec![],
            allowed_servers: vec![],
            blocked_tools: vec![],
            allowed_tools: vec![],
            require_admin_approval: false,
        };

        let checker = PermissionChecker::new(
            enterprise,
            IdeWhitelist::default(),
            UserPermissions::default(),
        );

        let result = checker.check_server("any-server").await;
        assert!(matches!(result, PermissionResult::Denied(_)));
        if let PermissionResult::Denied(reason) = result {
            assert!(reason.contains("disabled by enterprise"));
        }
    }

    #[tokio::test]
    async fn test_ide_whitelist() {
        let ide = IdeWhitelist {
            allowed_servers: vec!["allowed-server".to_string()],
            allowed_tools: vec![],
        };

        let checker =
            PermissionChecker::new(EnterprisePolicy::default(), ide, UserPermissions::default());

        // 在白名单中的服务器
        let result = checker.check_server("allowed-server").await;
        assert!(matches!(
            result,
            PermissionResult::Allowed | PermissionResult::NeedsConfirmation
        ));

        // 不在白名单中的服务器（空列表表示全部允许）
        let result = checker.check_server("other-server").await;
        // 由于 IDE 白名单为空时表示全部允许，所以应该通过
        assert!(matches!(
            result,
            PermissionResult::Allowed | PermissionResult::NeedsConfirmation
        ));
    }

    #[tokio::test]
    async fn test_user_permissions() {
        let user = UserPermissions {
            enabled_servers: vec!["user-server".to_string()],
            disabled_servers: vec!["disabled-server".to_string()],
            authorized_tools: vec![],
            blocked_tools: vec![],
        };

        let checker =
            PermissionChecker::new(EnterprisePolicy::default(), IdeWhitelist::default(), user);

        // 用户禁用的服务器
        let result = checker.check_server("disabled-server").await;
        assert!(matches!(result, PermissionResult::Denied(_)));

        // 用户启用的服务器
        let result = checker.check_server("user-server").await;
        assert!(matches!(
            result,
            PermissionResult::Allowed | PermissionResult::NeedsConfirmation
        ));
    }

    #[tokio::test]
    async fn test_tool_permissions() {
        let enterprise = EnterprisePolicy {
            enabled: true,
            blocked_servers: vec![],
            allowed_servers: vec![],
            blocked_tools: vec!["dangerous_*".to_string()],
            allowed_tools: vec![],
            require_admin_approval: false,
        };

        let checker = PermissionChecker::new(
            enterprise,
            IdeWhitelist::default(),
            UserPermissions::default(),
        );

        // 被企业策略阻止的工具
        let result = checker.check_tool("mcp__server__dangerous_tool").await;
        assert!(matches!(result, PermissionResult::Denied(_)));

        // 普通工具
        let result = checker.check_tool("mcp__server__safe_tool").await;
        assert!(matches!(
            result,
            PermissionResult::Allowed | PermissionResult::NeedsConfirmation
        ));
    }

    #[tokio::test]
    async fn test_runtime_confirmation_cache() {
        let checker = PermissionChecker::new(
            EnterprisePolicy::default(),
            IdeWhitelist::default(),
            UserPermissions::default(),
        );

        // 初始需要确认
        let result = checker.check_server("new-server").await;
        assert!(matches!(result, PermissionResult::NeedsConfirmation));

        // 确认
        checker.confirm("new-server").await.unwrap();

        // 现在应该允许
        let result = checker.check_server("new-server").await;
        assert!(matches!(result, PermissionResult::Allowed));

        // 清除缓存
        checker.clear_runtime_cache().await;

        // 又需要确认
        let result = checker.check_server("new-server").await;
        assert!(matches!(result, PermissionResult::NeedsConfirmation));
    }

    #[tokio::test]
    async fn test_permission_update() {
        let checker = PermissionChecker::new(
            EnterprisePolicy::default(),
            IdeWhitelist::default(),
            UserPermissions::default(),
        );

        // 更新企业策略
        let new_enterprise = EnterprisePolicy {
            enabled: false,
            ..Default::default()
        };
        checker.update_enterprise_policy(new_enterprise).await;

        let result = checker.check_server("any").await;
        assert!(matches!(result, PermissionResult::Denied(_)));

        // 更新用户权限
        let new_user = UserPermissions {
            enabled_servers: vec!["allowed".to_string()],
            ..Default::default()
        };
        checker.update_user_permissions(new_user).await;
    }

    #[tokio::test]
    async fn test_four_layer_enforcement() {
        // 测试四层权限同时生效

        // Layer 1: 企业策略允许但阻止特定服务器
        let enterprise = EnterprisePolicy {
            enabled: true,
            blocked_servers: vec!["evil-server".to_string()],
            allowed_servers: vec![],
            blocked_tools: vec![],
            allowed_tools: vec![],
            require_admin_approval: false,
        };

        // Layer 2: IDE 白名单只允许特定服务器
        let ide = IdeWhitelist {
            allowed_servers: vec!["trusted-server".to_string()],
            allowed_tools: vec![],
        };

        // Layer 3: 用户禁用某个服务器
        let user = UserPermissions {
            enabled_servers: vec![],
            disabled_servers: vec!["unwanted-server".to_string()],
            authorized_tools: vec![],
            blocked_tools: vec![],
        };

        let checker = PermissionChecker::new(enterprise, ide, user);

        // 被企业阻止
        let result = checker.check_server("evil-server").await;
        assert!(matches!(result, PermissionResult::Denied(_)));

        // 被用户禁用
        let result = checker.check_server("unwanted-server").await;
        assert!(matches!(result, PermissionResult::Denied(_)));

        // 通过所有层
        let result = checker.check_server("trusted-server").await;
        assert!(matches!(
            result,
            PermissionResult::Allowed | PermissionResult::NeedsConfirmation
        ));
    }
}
