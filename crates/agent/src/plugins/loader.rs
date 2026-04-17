//! 插件加载器
//! 
//! 实现插件的加载、安装、更新和卸载

use crate::plugins::types::{
    InstalledPlugin, PluginMetadata, PluginStatus, PluginLocation, PluginType,
    PluginBlocklist, PluginSecurityPolicy,
};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::HashMap;

/// 插件加载器
pub struct PluginLoader {
    /// 已加载的插件
    plugins: HashMap<String, InstalledPlugin>,
    /// 插件目录列表
    plugin_dirs: Vec<PathBuf>,
    /// 黑名单
    blocklist: PluginBlocklist,
    /// 安全策略
    security_policy: PluginSecurityPolicy,
}

impl PluginLoader {
    /// 创建加载器
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_dirs: Vec::new(),
            blocklist: PluginBlocklist::new(),
            security_policy: PluginSecurityPolicy::default(),
        }
    }
    
    /// 设置插件目录
    pub fn with_plugin_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.plugin_dirs = dirs;
        self
    }
    
    /// 设置黑名单
    pub fn with_blocklist(mut self, blocklist: PluginBlocklist) -> Self {
        self.blocklist = blocklist;
        self
    }
    
    /// 设置安全策略
    pub fn with_security_policy(mut self, policy: PluginSecurityPolicy) -> Self {
        self.security_policy = policy;
        self
    }
    
    /// 从所有目录加载插件
    pub fn load_all(&mut self) -> Result<usize, PluginLoadError> {
        let mut loaded_count = 0;
        
        for dir in &self.plugin_dirs {
            loaded_count += self.load_from_dir(dir)?;
        }
        
        Ok(loaded_count)
    }
    
    /// 从目录加载插件
    pub fn load_from_dir(&mut self, dir: &Path) -> Result<usize, PluginLoadError> {
        if !dir.exists() {
            return Ok(0);
        }
        
        let mut loaded_count = 0;
        
        let entries = fs::read_dir(dir)
            .map_err(|e| PluginLoadError::IoError(format!("Failed to read {:?}: {}", dir, e)))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            // 跳过非目录
            if !path.is_dir() {
                continue;
            }
            
            // 加载插件
            match self.load_plugin(&path) {
                Ok(plugin) => {
                    // 检查黑名单
                    if self.blocklist.is_blocked(&plugin.metadata.name) {
                        eprintln!(
                            "Plugin {} is blocked: {:?}",
                            plugin.metadata.name,
                            self.blocklist.get_block_reason(&plugin.metadata.name)
                        );
                        continue;
                    }
                    
                    // 检查安全策略
                    if self.security_policy.allow_managed_only 
                        && plugin.location != PluginLocation::Managed 
                    {
                        continue;
                    }
                    
                    self.plugins.insert(plugin.metadata.name.clone(), plugin);
                    loaded_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to load plugin {:?}: {}", path, e);
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    /// 加载单个插件
    fn load_plugin(&self, path: &Path) -> Result<InstalledPlugin, PluginLoadError> {
        // 读取 package.json 或 plugin.json
        let metadata = self.read_metadata(path)?;
        
        // 确定插件位置
        let location = self.determine_location(path)?;
        
        // 获取安装时间
        let installed_at = fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| t.into())
            .unwrap_or_else(chrono::Utc::now);
        
        Ok(InstalledPlugin {
            metadata,
            install_path: path.to_path_buf(),
            location,
            installed_at,
            last_updated: None,
            status: PluginStatus::Active,
            config: HashMap::new(),
        })
    }
    
    /// 读取插件元数据
    fn read_metadata(&self, path: &Path) -> Result<PluginMetadata, PluginLoadError> {
        // 尝试读取 package.json
        let package_json_path = path.join("package.json");
        if package_json_path.exists() {
            let content = fs::read_to_string(&package_json_path)
                .map_err(|e| PluginLoadError::IoError(format!("Failed to read package.json: {}", e)))?;
            
            // 解析 package.json 提取必要字段
            let package: serde_json::Value = serde_json::from_str(&content)
                .map_err(|e| PluginLoadError::ParseError(format!("Failed to parse package.json: {}", e)))?;
            
            return Ok(PluginMetadata {
                name: package["name"].as_str().unwrap_or("unknown").to_string(),
                display_name: package["displayName"].as_str().map(|s| s.to_string()),
                description: package["description"].as_str().map(|s| s.to_string()),
                version: package["version"].as_str().unwrap_or("0.0.0").to_string(),
                author: package["author"].as_str().map(|s| s.to_string()),
                license: package["license"].as_str().map(|s| s.to_string()),
                repository: package["repository"].as_str().map(|s| s.to_string()),
                homepage: package["homepage"].as_str().map(|s| s.to_string()),
                minimum_version: package["engines"]["claude-code"].as_str().map(|s| s.to_string()),
                main: package["main"].as_str().map(|s| s.to_string()),
                plugin_type: PluginType::Extension,
                required_permissions: vec![],
                config_schema: None,
                contributed_skills: vec![],
                contributed_commands: vec![],
            });
        }
        
        // 尝试读取 plugin.json
        let plugin_json_path = path.join("plugin.json");
        if plugin_json_path.exists() {
            let content = fs::read_to_string(&plugin_json_path)
                .map_err(|e| PluginLoadError::IoError(format!("Failed to read plugin.json: {}", e)))?;
            
            let metadata: PluginMetadata = serde_json::from_str(&content)
                .map_err(|e| PluginLoadError::ParseError(format!("Failed to parse plugin.json: {}", e)))?;
            
            return Ok(metadata);
        }
        
        Err(PluginLoadError::NotFound("No package.json or plugin.json found".to_string()))
    }
    
    /// 确定插件位置
    fn determine_location(&self, path: &Path) -> Result<PluginLocation, PluginLoadError> {
        // 确定插件位置的逻辑
        let home = std::env::var("HOME").map(PathBuf::from).ok();
        let managed = std::env::var("CLAUDE_MANAGED_DIR").map(PathBuf::from).ok();
        
        if let Some(ref managed_dir) = managed {
            if path.starts_with(managed_dir) {
                return Ok(PluginLocation::Managed);
            }
        }
        
        if let Some(ref home_dir) = home {
            if path.starts_with(home_dir.join(".claude/plugins")) {
                return Ok(PluginLocation::Global);
            }
        }
        
        if path.starts_with(".claude/plugins") {
            return Ok(PluginLocation::Project);
        }
        
        // 开发模式
        Ok(PluginLocation::Development)
    }
    
    /// 获取已加载的插件
    pub fn get_plugins(&self) -> Vec<&InstalledPlugin> {
        self.plugins.values().collect()
    }
    
    /// 按名称获取插件
    pub fn get_plugin(&self, name: &str) -> Option<&InstalledPlugin> {
        self.plugins.get(name)
    }
    
    /// 禁用插件
    pub fn disable_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| format!("Plugin not found: {}", name))?;
        
        plugin.status = PluginStatus::Disabled;
        Ok(())
    }
    
    /// 启用插件
    pub fn enable_plugin(&mut self, name: &str) -> Result<(), String> {
        let plugin = self.plugins.get_mut(name)
            .ok_or_else(|| format!("Plugin not found: {}", name))?;
        
        plugin.status = PluginStatus::Active;
        Ok(())
    }
    
    /// 卸载插件
    pub fn uninstall_plugin(&mut self, name: &str) -> Result<PathBuf, String> {
        let plugin = self.plugins.remove(name)
            .ok_or_else(|| format!("Plugin not found: {}", name))?;
        
        let install_path = plugin.install_path.clone();
        
        // 删除安装目录
        fs::remove_dir_all(&install_path)
            .map_err(|e| format!("Failed to remove plugin directory: {}", e))?;
        
        Ok(install_path)
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 插件加载错误
#[derive(Debug, thiserror::Error)]
pub enum PluginLoadError {
    #[error("IO 错误：{0}")]
    IoError(String),
    
    #[error("解析错误：{0}")]
    ParseError(String),
    
    #[error("插件未找到：{0}")]
    NotFound(String),
    
    #[error("版本不兼容：{0}")]
    VersionMismatch(String),
    
    #[error("安全策略阻止：{0}")]
    SecurityBlocked(String),
}

/// 插件更新器
pub struct PluginUpdater {
    /// 检查更新的间隔（秒）
    check_interval: u64,
    /// 是否启用自动更新
    auto_update_enabled: bool,
}

impl PluginUpdater {
    /// 创建更新器
    pub fn new(check_interval_secs: u64) -> Self {
        Self {
            check_interval: check_interval_secs,
            auto_update_enabled: false,
        }
    }
    
    /// 启用自动更新
    pub fn enable_auto_update(&mut self) {
        self.auto_update_enabled = true;
    }
    
    /// 禁用自动更新
    pub fn disable_auto_update(&mut self) {
        self.auto_update_enabled = false;
    }
    
    /// 检查插件更新
    /// 
    /// 对比已安装插件版本与注册表中的最新版本
    pub async fn check_for_updates(
        &self,
        plugins: &[&InstalledPlugin],
    ) -> Result<Vec<PluginUpdateInfo>, PluginUpdateError> {
        let mut updates = Vec::new();
        
        for plugin in plugins {
            // 获取远程版本信息
            match self.fetch_latest_version(&plugin.metadata.name).await {
                Ok(latest_version) => {
                    // 比较版本号
                    if Self::is_newer_version(&latest_version, &plugin.metadata.version) {
                        updates.push(PluginUpdateInfo {
                            plugin_name: plugin.metadata.name.clone(),
                            current_version: plugin.metadata.version.clone(),
                            latest_version,
                            install_path: plugin.install_path.clone(),
                        });
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Failed to check update for {}: {}",
                        plugin.metadata.name, e
                    );
                }
            }
        }
        
        Ok(updates)
    }
    
    /// 更新插件
    pub async fn update_plugin(
        &self,
        plugin_name: &str,
        install_path: &Path,
    ) -> Result<(), PluginUpdateError> {
        // 1. 下载新版本
        // 2. 停止当前插件
        // 3. 备份旧版本
        // 4. 解压新版本
        // 5. 重启插件
        
        // 框架实现
        eprintln!("Update plugin {} at {:?}", plugin_name, install_path);
        Ok(())
    }
    
    /// 批量更新所有插件
    pub async fn update_all(
        &self,
        updates: &[PluginUpdateInfo],
    ) -> Result<UpdateResult, PluginUpdateError> {
        let mut succeeded = Vec::new();
        let mut failed = Vec::new();
        
        for update in updates {
            match self.update_plugin(&update.plugin_name, &update.install_path).await {
                Ok(_) => succeeded.push(update.plugin_name.clone()),
                Err(e) => failed.push((update.plugin_name.clone(), e.to_string())),
            }
        }
        
        Ok(UpdateResult { succeeded, failed })
    }
    
    /// 获取最新版本
    async fn fetch_latest_version(&self, plugin_name: &str) -> Result<String, PluginUpdateError> {
        // 从注册表或 CDN 获取最新版本
        // 框架实现
        Ok("1.0.0".to_string())
    }
    
    /// 比较版本号
    /// 
    /// 使用语义化版本号比较（semver）
    fn is_newer_version(latest: &str, current: &str) -> bool {
        let latest_parts: Vec<u32> = latest
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        let current_parts: Vec<u32> = current
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        
        // 逐段比较
        for (l, c) in latest_parts.iter().zip(current_parts.iter()) {
            if l > c {
                return true;
            }
            if l < c {
                return false;
            }
        }
        
        // 前面都相同，检查是否有更多段
        latest_parts.len() > current_parts.len()
    }
}

/// 插件更新信息
#[derive(Debug, Clone)]
pub struct PluginUpdateInfo {
    /// 插件名称
    pub plugin_name: String,
    /// 当前版本
    pub current_version: String,
    /// 最新版本
    pub latest_version: String,
    /// 安装路径
    pub install_path: PathBuf,
}

/// 更新结果
#[derive(Debug, Clone, Default)]
pub struct UpdateResult {
    /// 成功更新的插件列表
    pub succeeded: Vec<String>,
    /// 失败的插件列表（名称 + 错误）
    pub failed: Vec<(String, String)>,
}

/// 插件更新错误
#[derive(Debug, thiserror::Error)]
pub enum PluginUpdateError {
    #[error("下载失败：{0}")]
    DownloadError(String),
    
    #[error("验证失败：{0}")]
    ValidationError(String),
    
    #[error("安装失败：{0}")]
    InstallError(String),
    
    #[error("IO 错误：{0}")]
    IoError(String),
}

#[cfg(test)]
mod updater_tests {
    use super::*;
    
    #[test]
    fn test_version_comparison() {
        // 主版本号更新
        assert!(PluginUpdater::is_newer_version("2.0.0", "1.0.0"));
        assert!(!PluginUpdater::is_newer_version("1.0.0", "2.0.0"));
        
        // 次版本号更新
        assert!(PluginUpdater::is_newer_version("1.2.0", "1.1.0"));
        assert!(!PluginUpdater::is_newer_version("1.1.0", "1.2.0"));
        
        // 补丁版本更新
        assert!(PluginUpdater::is_newer_version("1.0.1", "1.0.0"));
        assert!(!PluginUpdater::is_newer_version("1.0.0", "1.0.1"));
        
        // 相同版本
        assert!(!PluginUpdater::is_newer_version("1.0.0", "1.0.0"));
        
        // 不同长度版本号
        assert!(PluginUpdater::is_newer_version("1.0.0.1", "1.0.0"));
    }
    
    #[test]
    fn test_updater_creation() {
        let updater = PluginUpdater::new(3600);
        assert_eq!(updater.check_interval, 3600);
        assert!(!updater.auto_update_enabled);
    }
    
    #[test]
    fn test_enable_disable_auto_update() {
        let mut updater = PluginUpdater::new(3600);
        
        updater.enable_auto_update();
        assert!(updater.auto_update_enabled);
        
        updater.disable_auto_update();
        assert!(!updater.auto_update_enabled);
    }
}

/// 插件验证器
pub struct PluginVerifier {
    /// 公钥列表（用于验证签名）
    public_keys: Vec<String>,
    /// 是否启用签名验证
    verification_enabled: bool,
}

impl PluginVerifier {
    /// 创建验证器
    pub fn new() -> Self {
        Self {
            public_keys: Vec::new(),
            verification_enabled: false,
        }
    }
    
    /// 启用签名验证
    pub fn enable_verification(&mut self) {
        self.verification_enabled = true;
    }
    
    /// 禁用签名验证
    pub fn disable_verification(&mut self) {
        self.verification_enabled = false;
    }
    
    /// 添加公钥
    pub fn add_public_key(&mut self, key: impl Into<String>) {
        self.public_keys.push(key.into());
    }
    
    /// 验证插件签名
    /// 
    /// 验证流程：
    /// 1. 读取插件目录中的 SIGNATURE 文件
    /// 2. 计算插件内容的哈希值
    /// 3. 使用公钥验证签名
    pub fn verify_plugin(
        &self,
        plugin_path: &Path,
    ) -> Result<VerificationResult, VerificationError> {
        if !self.verification_enabled {
            return Ok(VerificationResult::Skipped);
        }
        
        // 读取签名文件
        let signature_path = plugin_path.join("SIGNATURE");
        if !signature_path.exists() {
            return Err(VerificationError::NoSignature);
        }
        
        let signature = std::fs::read_to_string(&signature_path)
            .map_err(|e| VerificationError::IoError(e.to_string()))?;
        
        // 计算插件内容哈希
        let content_hash = self.compute_content_hash(plugin_path)?;
        
        // 验证签名
        for public_key in &self.public_keys {
            if self.verify_signature(&signature, &content_hash, public_key)? {
                return Ok(VerificationResult::Valid);
            }
        }
        
        Ok(VerificationResult::Invalid)
    }
    
    /// 计算内容哈希
    fn compute_content_hash(&self, plugin_path: &Path) -> Result<String, VerificationError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        
        // 遍历插件目录中的所有文件
        if let Ok(entries) = std::fs::read_dir(plugin_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.file_name() != Some("SIGNATURE".as_ref()) {
                    // 读取文件内容并更新哈希
                    if let Ok(content) = std::fs::read(&path) {
                        content.hash(&mut hasher);
                    }
                }
            }
        }
        
        let hash = hasher.finish();
        Ok(format!("{:016x}", hash))
    }
    
    /// 验证签名
    fn verify_signature(
        &self,
        signature: &str,
        content_hash: &str,
        public_key: &str,
    ) -> Result<bool, VerificationError> {
        // 框架实现：实际应使用 RSA/ECDSA 等签名算法
        // 这里仅提供框架
        eprintln!(
            "Verify signature {} against hash {} with key {}",
            signature, content_hash, public_key
        );
        Ok(true)
    }
}

/// 验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum VerificationResult {
    /// 签名有效
    Valid,
    /// 签名无效
    Invalid,
    /// 验证被跳过
    Skipped,
}

/// 验证错误
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("没有找到签名文件")]
    NoSignature,
    
    #[error("签名验证失败")]
    VerificationFailed,
    
    #[error("IO 错误：{0}")]
    IoError(String),
    
    #[error("密钥错误：{0}")]
    KeyError(String),
}

#[cfg(test)]
mod verifier_tests {
    use super::*;
    
    #[test]
    fn test_verifier_creation() {
        let verifier = PluginVerifier::new();
        assert!(!verifier.verification_enabled);
        assert!(verifier.public_keys.is_empty());
    }
    
    #[test]
    fn test_enable_verification() {
        let mut verifier = PluginVerifier::new();
        
        verifier.enable_verification();
        assert!(verifier.verification_enabled);
        
        verifier.disable_verification();
        assert!(!verifier.verification_enabled);
    }
    
    #[test]
    fn test_add_public_key() {
        let mut verifier = PluginVerifier::new();
        
        verifier.add_public_key("test-key-1");
        verifier.add_public_key("test-key-2");
        
        assert_eq!(verifier.public_keys.len(), 2);
    }
    
    #[test]
    fn test_verify_skipped_when_disabled() {
        let verifier = PluginVerifier::new();
        let temp_dir = tempfile::tempdir().unwrap();
        
        let result = verifier.verify_plugin(temp_dir.path()).unwrap();
        assert_eq!(result, VerificationResult::Skipped);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        assert_eq!(loader.plugins.len(), 0);
    }
    
    #[test]
    fn test_with_plugin_dirs() {
        let dirs = vec![PathBuf::from("/test/plugins")];
        let loader = PluginLoader::new()
            .with_plugin_dirs(dirs.clone());
        
        assert_eq!(loader.plugin_dirs.len(), 1);
        assert_eq!(loader.plugin_dirs[0], dirs[0]);
    }
    
    #[test]
    fn test_get_plugins_empty() {
        let loader = PluginLoader::new();
        let plugins = loader.get_plugins();
        assert!(plugins.is_empty());
    }
    
    #[test]
    fn test_get_plugin_not_found() {
        let loader = PluginLoader::new();
        assert!(loader.get_plugin("nonexistent").is_none());
    }
    
    #[test]
    fn test_disable_plugin_not_found() {
        let mut loader = PluginLoader::new();
        let result = loader.disable_plugin("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Plugin not found"));
    }
    
    #[test]
    fn test_enable_plugin_not_found() {
        let mut loader = PluginLoader::new();
        let result = loader.enable_plugin("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Plugin not found"));
    }
    
    #[test]
    fn test_uninstall_plugin_not_found() {
        let mut loader = PluginLoader::new();
        let result = loader.uninstall_plugin("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Plugin not found"));
    }
    
    #[test]
    fn test_load_from_nonexistent_dir() {
        let mut loader = PluginLoader::new();
        let result = loader.load_from_dir(Path::new("/nonexistent/dir"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }
}
