//! User Context Module
//!
//! Implements user-level context injection aligned with Claude Code's context.ts

use super::memory::MemoryDir;
use super::memory::truncate_entrypoint;
use super::memory_files::discover_memory_files;
use chrono::Local;

#[derive(Debug, Clone)]
pub struct UserContext {
    pub memory_files: Option<String>,
    pub memory_md_content: Option<String>,
    pub current_date: String,
}

pub struct UserContextProvider;

impl UserContextProvider {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_context(&self) -> UserContext {
        let memory_files = discover_memory_files();
        let memory_content = memory_files.map(|files| {
            files
                .iter()
                .map(|f| f.content.clone())
                .collect::<Vec<_>>()
                .join("\n\n")
        });

        let memory_md_content = get_memory_md_content().await;

        UserContext {
            memory_files: memory_content,
            memory_md_content,
            current_date: get_local_iso_date(),
        }
    }
}

impl Default for UserContextProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub fn get_local_iso_date() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

async fn get_memory_md_content() -> Option<String> {
    let dir = MemoryDir::resolve();
    let index_path = dir.index_path();
    
    if !index_path.exists() {
        return None;
    }
    
    let content = tokio::fs::read_to_string(&index_path).await.ok()?;
    let truncation = truncate_entrypoint(&content);
    
    if truncation.was_truncated() {
        let warning = format!(
            "\n\n> WARNING: MEMORY.md is {}. Only part of it was loaded. Keep index entries to one line under ~200 chars; move detail into topic files.",
            truncation.truncation_reason()
        );
        Some(truncation.content + &warning)
    } else {
        Some(truncation.content)
    }
}

pub fn get_user_context() -> UserContext {
    let memory_files = crate::context::memory_files::discover_memory_files();
    let memory_content = memory_files.map(|files| {
        files
            .iter()
            .map(|f| f.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    });

    let memory_md_content = futures::executor::block_on(get_memory_md_content());

    UserContext {
        memory_files: memory_content,
        memory_md_content,
        current_date: get_local_iso_date(),
    }
}

pub async fn get_user_context_async() -> UserContext {
    UserContextProvider::new().get_context().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_local_iso_date_format() {
        let date = get_local_iso_date();
        assert_eq!(date.len(), 10);
        assert!(date.contains('-'));
    }

    #[test]
    fn test_user_context_provider_creation() {
        let ctx = get_user_context();
        assert!(ctx.current_date.len() == 10);
    }

    #[tokio::test]
    async fn test_user_context_async() {
        let provider = UserContextProvider::new();
        let ctx = provider.get_context().await;
        assert!(ctx.current_date.len() == 10);
    }
}
