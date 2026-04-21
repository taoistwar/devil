//! Memory Files Discovery Module
//!
//! Implements CLAUDE.md file discovery aligned with Claude Code's context.ts

use std::fs;
use std::path::{Path, PathBuf};

const CLAUDE_MD_FILENAME: &str = "CLAUDE.md";

#[derive(Debug, Clone)]
pub struct MemoryFile {
    pub path: PathBuf,
    pub content: String,
}

pub struct MemoryFilesCollector {
    cwd: PathBuf,
    disable_claude_mds: bool,
    is_bare_mode: bool,
    additional_dirs: Vec<PathBuf>,
}

impl MemoryFilesCollector {
    pub fn new() -> Self {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self {
            cwd,
            disable_claude_mds: std::env::var("CLAUDE_CODE_DISABLE_CLAUDE_MDS")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            is_bare_mode: std::env::var("CLAUDE_CODE_BARE")
                .map(|v| v == "1" || v.to_lowercase() == "true")
                .unwrap_or(false),
            additional_dirs: Self::get_additional_directories(),
        }
    }

    fn get_additional_directories() -> Vec<PathBuf> {
        std::env::var("CLAUDE_CODE_ADD_DIR")
            .map(|v| {
                v.split(',')
                    .filter_map(|s| {
                        let path = PathBuf::from(s.trim());
                        if path.exists() && path.is_dir() {
                            Some(path)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn discover(&self) -> Option<Vec<MemoryFile>> {
        if self.disable_claude_mds && self.additional_dirs.is_empty() {
            return None;
        }

        if self.is_bare_mode && self.additional_dirs.is_empty() {
            return None;
        }

        let mut files = Vec::new();

        if self.is_bare_mode && !self.additional_dirs.is_empty() {
            for dir in &self.additional_dirs {
                if let Ok(claude_md) = self.read_claude_md(dir) {
                    files.push(claude_md);
                }
            }
        } else {
            let mut current = self.cwd.clone();
            loop {
                if let Ok(claude_md) = self.read_claude_md(&current) {
                    files.push(claude_md);
                }

                if !current.pop() {
                    break;
                }
            }
        }

        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }

    fn read_claude_md(&self, dir: &Path) -> Result<MemoryFile, std::io::Error> {
        let path = dir.join(CLAUDE_MD_FILENAME);
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            Ok(MemoryFile { path, content })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "CLAUDE.md not found",
            ))
        }
    }

    pub fn combine_contents(&self, files: &[MemoryFile]) -> String {
        files
            .iter()
            .map(|f| f.content.clone())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Default for MemoryFilesCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub fn discover_memory_files() -> Option<Vec<MemoryFile>> {
    MemoryFilesCollector::new().discover()
}

pub fn clear_memory_files_cache() {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::TempDir;

    #[test]
    fn test_memory_file_collector_creation() {
        let collector = MemoryFilesCollector::new();
        assert!(collector.cwd.exists() || collector.cwd.to_str().is_some());
    }

    #[test]
    fn test_read_nonexistent_claude_md() {
        let collector = MemoryFilesCollector::new();
        let temp_dir = TempDir::new().unwrap();
        let result = collector.read_claude_md(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_combine_contents_empty() {
        let collector = MemoryFilesCollector::new();
        let files: Vec<MemoryFile> = vec![];
        assert_eq!(collector.combine_contents(&files), "");
    }

    #[test]
    fn test_disable_claude_mds_env() {
        env::set_var("CLAUDE_CODE_DISABLE_CLAUDE_MDS", "1");
        let collector = MemoryFilesCollector::new();
        assert!(collector.disable_claude_mds);
        env::remove_var("CLAUDE_CODE_DISABLE_CLAUDE_MDS");
    }
}
