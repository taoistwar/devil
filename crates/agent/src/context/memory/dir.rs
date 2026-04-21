//! Memory Directory Management
//!
//! Handles memory directory path resolution following Claude Code's patterns:
//! 1. CLAUDE_COWORK_MEMORY_PATH_OVERRIDE env var (full override)
//! 2. autoMemoryDirectory in settings.json (user/policy/local only)
//! 3. <memoryBase>/projects/<sanitized-git-root>/memory/

use crate::context::memory::index::{IndexEntry, MemoryIndex};
use crate::context::memory::types::{MemoryEntry, MemoryType};
use std::path::{Path, PathBuf};

/// Memory directory configuration and operations
#[derive(Debug, Clone)]
pub struct MemoryDir {
    path: PathBuf,
}

impl MemoryDir {
    /// Resolve memory directory from environment/settings
    pub fn resolve() -> Self {
        let path = if let Some(override_path) = std::env::var("CLAUDE_COWORK_MEMORY_PATH_OVERRIDE")
            .ok()
            .filter(|p| !p.is_empty())
        {
            PathBuf::from(override_path)
        } else {
            Self::default_memory_path()
        };

        Self { path }
    }

    /// Get default memory path: ~/.claude/projects/<project>/memory/
    fn default_memory_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let project_name = sanitize_project_name(&cwd);
        home.join(".claude")
            .join("projects")
            .join(project_name)
            .join("memory")
    }

    /// Get the memory directory path
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Get MEMORY.md index path
    pub fn index_path(&self) -> PathBuf {
        self.path.join("MEMORY.md")
    }

    /// Get path for a new memory file
    pub fn memory_path(&self, memory_type: MemoryType, name: &str) -> PathBuf {
        let safe_name = sanitize_name(name);
        let filename = format!("{}{}.md", memory_type.file_prefix(), safe_name);
        self.path.join(filename)
    }

    /// List all memory files in the directory
    pub async fn list_memories(&self) -> anyhow::Result<Vec<MemoryEntry>> {
        let mut entries = Vec::new();

        if !self.path.exists() {
            return Ok(entries);
        }

        let mut dir = tokio::fs::read_dir(&self.path).await?;

        while let Some(item) = dir.next_entry().await? {
            let path = item.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Ok(entry) = crate::context::memory::types::parse_memory_file(&path) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    /// List memory files filtered by type
    pub async fn list_memories_by_type(&self, memory_type: MemoryType) -> anyhow::Result<Vec<MemoryEntry>> {
        let all = self.list_memories().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.frontmatter.memory_type == memory_type)
            .collect())
    }

    /// Ensure memory directory exists
    pub async fn ensure_exists(&self) -> anyhow::Result<()> {
        if !self.path.exists() {
            tokio::fs::create_dir_all(&self.path).await?;
        }
        Ok(())
    }

    /// Save a memory entry (writes file and updates index)
    pub async fn save_memory(&self, entry: &MemoryEntry) -> anyhow::Result<()> {
        self.ensure_exists().await?;

        let file_path = &entry.path;
        let content = format!(
            "---\nname: {}\ndescription: {}\ntype: {}\n---\n{}\n",
            entry.frontmatter.name,
            entry.frontmatter.description,
            entry.frontmatter.memory_type,
            entry.content
        );

        tokio::fs::write(file_path, content).await?;

        self.update_index_add(&entry.frontmatter.name, file_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(""),
            &entry.frontmatter.description).await?;

        Ok(())
    }

    /// Load all memories from the memory directory
    pub async fn load_memories(&self) -> anyhow::Result<Vec<MemoryEntry>> {
        if !self.path.exists() {
            return Ok(Vec::new());
        }
        self.list_memories().await
    }

    /// Update index by adding an entry
    async fn update_index_add(&self, title: &str, filename: &str, hook: &str) -> anyhow::Result<()> {
        let index_path = self.index_path();
        let content = if index_path.exists() {
            tokio::fs::read_to_string(&index_path).await?
        } else {
            String::new()
        };

        let mut index = MemoryIndex::parse(&content);
        index.upsert(IndexEntry {
            title: title.to_string(),
            file: filename.to_string(),
            hook: hook.to_string(),
        });

        tokio::fs::write(&index_path, index.to_content()).await?;
        Ok(())
    }

    /// Delete a memory by path
    pub async fn delete_memory(&self, path: &Path) -> anyhow::Result<()> {
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }

        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            let index_path = self.index_path();
            if index_path.exists() {
                let content = tokio::fs::read_to_string(&index_path).await?;
                let mut index = MemoryIndex::parse(&content);
                index.remove(filename);
                tokio::fs::write(&index_path, index.to_content()).await?;
            }
        }
        Ok(())
    }
}

/// Sanitize project name for path
fn sanitize_project_name(path: &Path) -> String {
    path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Sanitize memory name for file path
fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_project_name() {
        let path = PathBuf::from("/home/user/my-project");
        assert_eq!(sanitize_project_name(&path), "my-project");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("user role"), "userrole");
        assert_eq!(sanitize_name("test-memory!"), "test-memory");
    }

    #[tokio::test]
    async fn test_memory_dir_resolve() {
        let dir = MemoryDir::resolve();
        assert!(dir.path().components().count() >= 3);
    }

    #[tokio::test]
    async fn test_memory_path_generation() {
        let dir = MemoryDir::resolve();
        let path = dir.memory_path(MemoryType::User, "test memory");
        let filename = path.file_name().unwrap().to_str().unwrap();
        assert!(filename.starts_with("user_"));
        assert!(filename.ends_with(".md"));
    }
}
