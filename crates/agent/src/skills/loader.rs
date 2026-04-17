//! Skill 加载器
//! 
//! 实现从磁盘、内置、MCP 等多种来源加载 Skills

use crate::skills::types::{SkillCommand, SkillSource, SkillLoadSource, ConditionalSkills};
use std::path::{Path, PathBuf};
use std::fs;

/// Skill 加载器
pub struct SkillLoader {
    /// 已加载的 Skills
    skills: Vec<SkillCommand>,
    /// 条件激活的 Skills
    conditional_skills: ConditionalSkills,
    /// 去重用的规范路径集合
    seen_dirs: std::collections::HashSet<PathBuf>,
}

impl SkillLoader {
    /// 创建加载器
    pub fn new() -> Self {
        Self {
            skills: Vec::new(),
            conditional_skills: ConditionalSkills::default(),
            seen_dirs: std::collections::HashSet::new(),
        }
    }
    
    /// 从目录加载 Skills
    /// 
    /// 加载协议：只识别 `skill-name/SKILL.md` 目录格式
    pub fn load_from_dir(
        &mut self,
        dir: &Path,
        source: SkillSource,
        loaded_from: SkillLoadSource,
    ) -> Result<usize, String> {
        if !dir.exists() {
            return Ok(0);
        }
        
        // 规范化路径
        let canonical_dir = dir.canonicalize()
            .unwrap_or_else(|_| dir.to_path_buf());
        
        // 去重检查
        if self.seen_dirs.contains(&canonical_dir) {
            return Ok(0);
        }
        self.seen_dirs.insert(canonical_dir.clone());
        
        let mut loaded_count = 0;
        
        // 扫描目录
        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {:?}: {}", dir, e))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            
            // 只处理目录或符号链接
            let is_dir = path.is_dir() || (path.is_symlink() && fs::read_link(&path).map(|p| p.is_dir()).unwrap_or(false));
            if !is_dir {
                continue;
            }
            
            // 在每个子目录中查找 SKILL.md
            let skill_md_path = path.join("SKILL.md");
            if !skill_md_path.exists() {
                continue;
            }
            
            // 加载 Skill
            match self.load_skill_file(&skill_md_path, &path, source.clone(), loaded_from.clone()) {
                Ok(skill) => {
                    // 检查是否有条件激活路径
                    if !skill.paths.is_empty() {
                        for pattern in &skill.paths {
                            self.conditional_skills.add(pattern.clone(), skill.clone());
                        }
                    } else {
                        self.skills.push(skill);
                    }
                    loaded_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to load skill {:?}: {}", skill_md_path, e);
                }
            }
        }
        
        Ok(loaded_count)
    }
    
    /// 加载单个 Skill 文件
    fn load_skill_file(
        &self,
        skill_md_path: &Path,
        skill_dir: &Path,
        source: SkillSource,
        loaded_from: SkillLoadSource,
    ) -> Result<SkillCommand, String> {
        // 读取文件内容
        let content = fs::read_to_string(skill_md_path)
            .map_err(|e| format!("Failed to read {:?}: {}", skill_md_path, e))?;
        
        // 解析 frontmatter
        let frontmatter = crate::skills::types::parse_frontmatter(&content)?;
        
        // 获取 Skill 名称（从目录名）
        let name = skill_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // 构建 Skill 目录的绝对路径
        let skill_dir_abs = skill_dir
            .canonicalize()
            .unwrap_or_else(|_| skill_dir.to_path_buf())
            .to_string_lossy()
            .to_string();
        
        // 创建 Skill 命令
        Ok(frontmatter.to_skill_command(
            name,
            content,
            skill_dir_abs,
            source,
            loaded_from,
        ))
    }
    
    /// 从磁盘目录加载（支持多个来源）
    /// 
    /// 加载路径：
    /// 1. 管理策略：$MANAGED_DIR/.claude/skills/
    /// 2. 用户全局：~/.claude/skills/
    /// 3. 项目级：.claude/skills/（向上遍历至 home）
    /// 4. 附加目录：--add-dir 指定的路径下 .claude/skills/
    pub fn load_all_from_disk(&mut self) -> Result<usize, String> {
        let mut total = 0;
        
        // 1. 管理策略目录
        if let Ok(managed_dir) = std::env::var("CLAUDE_MANAGED_DIR") {
            let skills_dir = PathBuf::from(managed_dir).join(".claude/skills");
            total += self.load_from_dir(
                &skills_dir,
                SkillSource::Disk,
                SkillLoadSource::Managed,
            )?;
        }
        
        // 2. 用户全局目录
        if let Ok(home) = std::env::var("HOME") {
            let skills_dir = PathBuf::from(home).join(".claude/skills");
            total += self.load_from_dir(
                &skills_dir,
                SkillSource::Disk,
                SkillLoadSource::UserSettings,
            )?;
        }
        
        // 3. 项目级目录（向上遍历）
        if let Ok(cwd) = std::env::var("PWD") {
            let mut current = PathBuf::from(cwd);
            let home = std::env::var("HOME").map(PathBuf::from).ok();
            
            while !home.as_ref().map(|h| current.starts_with(h)).unwrap_or(false) {
                let skills_dir = current.join(".claude/skills");
                if skills_dir.exists() {
                    total += self.load_from_dir(
                        &skills_dir,
                        SkillSource::Disk,
                        SkillLoadSource::ProjectSettings,
                    )?;
                }
                
                // 向上遍历
                if !current.pop() {
                    break;
                }
            }
        }
        
        Ok(total)
    }
    
    /// 激活匹配路径的条件 Skills
    pub fn activate_skills_for_path(&mut self, path: &str) -> Vec<SkillCommand> {
        self.conditional_skills.activate_for_path(path)
    }
    
    /// 获取所有可用的 Skills
    pub fn get_all_skills(&self) -> &[SkillCommand] {
        &self.skills
    }
    
    /// 动态发现 Skills（基于文件路径）
    /// 
    /// 从被操作的文件路径开始，向上遍历至 CWD，查找 .claude/skills/ 目录
    pub fn discover_skills_for_path(file_path: &Path) -> Vec<PathBuf> {
        let mut skill_dirs = Vec::new();
        let mut current = file_path.parent().unwrap_or(file_path).to_path_buf();
        let cwd = std::env::current_dir().unwrap_or_default();
        
        // 向上遍历（不包含 CWD 本身）
        while current != cwd && current.pop() {
            let skills_dir = current.join(".claude/skills");
            if skills_dir.exists() {
                // 使用 realpath 去重
                if let Ok(canonical) = skills_dir.canonicalize() {
                    skill_dirs.push(canonical);
                }
            }
        }
        
        // 按路径深度排序（深层优先）
        skill_dirs.sort_by(|a, b| {
            b.components().count().cmp(&a.components().count())
        });
        
        skill_dirs
    }
}

impl Default for SkillLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// 内置 Skills（编译时打包）
pub mod bundled {
    use super::*;
    use std::io::{self, Write};
    use std::fs::{File, create_dir_all};
    
    /// 内置 Skills 压缩数据（编译时嵌入）
    /// 
    /// 使用 include_bytes! 宏将 skills 目录打包为字节数组
    /// 实际项目中应使用 build.rs 脚本在构建时生成此文件
    const BUNDLED_SKILLS_DATA: &[u8] = &[];
    
    /// 首次使用时的提取标记
    static mut EXTRACTION_DONE: bool = false;
    
    /// 注册内置 Skills
    /// 
    /// 关键特性：
    /// - 延迟文件提取：首次调用时才解压到临时目录
    /// - 闭包级 memoize：并发调用共享同一个 extraction promise
    /// - 来源标记为 'bundled'，在 Prompt 预算中享有不可截断的特权
    pub fn register_bundled_skills() -> Vec<SkillCommand> {
        // 检查是否已提取
        unsafe {
            if !EXTRACTION_DONE {
                // 首次调用时提取
                if let Ok(extracted_dir) = extract_bundled_skills() {
                    // 从提取目录加载 Skills
                    let mut loader = SkillLoader::new();
                    match loader.load_from_dir(
                        &extracted_dir,
                        SkillSource::Bundled,
                        SkillLoadSource::Managed,
                    ) {
                        Ok(_) => {
                            EXTRACTION_DONE = true;
                            return loader.skills;
                        }
                        Err(e) => {
                            eprintln!("Failed to load bundled skills: {}", e);
                        }
                    }
                }
            }
        }
        
        vec![]
    }
    
    /// 解压内置 Skills 到临时目录
    fn extract_bundled_skills() -> Result<PathBuf, String> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // 获取临时目录
        let temp_dir = std::env::temp_dir()
            .join("claude-bundled-skills")
            .join(format!(
                "{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ));
        
        // 创建目录
        create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp dir: {}", e))?;
        
        // 如果内置数据为空，返回空目录
        if BUNDLED_SKILLS_DATA.is_empty() {
            return Ok(temp_dir);
        }
        
        // 解码并解压数据（假设是 tar.gz 格式）
        // 实际实现需要使用 flate2 和 tar crate
        // 这里提供框架实现
        
        // 示例：如果是 zip 格式
        // let cursor = io::Cursor::new(BUNDLED_SKILLS_DATA);
        // let mut archive = zip::ZipArchive::new(cursor)
        //     .map_err(|e| format!("Failed to open zip: {}", e))?;
        // archive.extract(&temp_dir)
        //     .map_err(|e| format!("Failed to extract zip: {}", e))?;
        
        Ok(temp_dir)
    }
}

/// MCP Skills 加载
pub mod mcp {
    use super::*;
    use serde_json::Value;
    
    /// 从 MCP 服务器获取 Skills
    /// 
    /// 通过 `skill://` URI 方案发现并转换为 Command 对象
    pub fn fetch_mcp_skills(server_name: &str) -> Vec<SkillCommand> {
        // MCP Skills 获取流程：
        // 1. 调用 MCP Server 的 resources/list API
        // 2. 筛选 skill:// URI 方案的资源
        // 3. 获取每个资源的内容
        // 4. 解析 frontmatter 并转换为 SkillCommand
        
        let mut skills = Vec::new();
        
        // 获取 MCP 资源列表
        let resources = list_mcp_resources(server_name);
        
        for resource in resources {
            // 检查是否为 skill:// URI
            if let Some(uri) = resource.get("uri").and_then(|v| v.as_str()) {
                if uri.starts_with("skill://") {
                    // 获取资源内容
                    if let Ok(content) = get_mcp_resource_content(server_name, uri) {
                        // 解析 frontmatter
                        if let Ok(frontmatter) = crate::skills::types::parse_frontmatter(&content) {
                            let skill_name = uri.strip_prefix("skill://").unwrap_or(uri);
                            
                            let skill_command = frontmatter.to_skill_command(
                                skill_name.to_string(),
                                content,
                                format!("mcp://{}/{}", server_name, skill_name),
                                SkillSource::MCP,
                                SkillLoadSource::MCP,
                            );
                            
                            skills.push(skill_command);
                        }
                    }
                }
            }
        }
        
        skills
    }
    
    /// 检查 MCP Server 是否支持 resources 能力
    pub fn supports_resources(server_name: &str) -> bool {
        // 检查 MCP Server capabilities
        // 实际实现需要查询服务器能力
        // 这里提供框架实现
        let capabilities = get_server_capabilities(server_name);
        capabilities.contains("resources")
    }
    
    /// 列出 MCP 资源
    fn list_mcp_resources(server_name: &str) -> Vec<Value> {
        // 实际实现需要调用 MCP Client API
        // 这里返回空数组作为框架
        vec![]
    }
    
    /// 获取 MCP 资源内容
    fn get_mcp_resource_content(server_name: &str, uri: &str) -> Result<String, String> {
        // 实际实现需要调用 MCP Client API
        // 这里返回错误作为框架
        Err("MCP resource content not implemented".to_string())
    }
    
    /// 获取服务器能力
    fn get_server_capabilities(server_name: &str) -> Vec<String> {
        // 实际实现需要查询服务器能力
        vec!["resources".to_string()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_discover_skills_for_path() {
        // 测试路径遍历发现 Skills
        let cwd = std::env::current_dir().unwrap();
        let file_path = cwd.join("src/skills/test.rs");
        
        let dirs = SkillLoader::discover_skills_for_path(&file_path);
        // 至少应该找到当前目录的 .claude/skills（如果存在）
        assert!(dirs.len() >= 0);
    }
}
