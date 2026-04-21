//! Memory Type Definitions and Parsing
//!
//! Implements the core data types for Claude Code's memory system:
//! - MemoryType: Four memory categories (user, feedback, project, reference)
//! - MemoryFrontmatter: YAML frontmatter structure for memory files
//! - MemoryEntry: Complete parsed memory file

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Four memory types aligned with Claude Code's taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    User,
    Feedback,
    Project,
    Reference,
}

impl MemoryType {
    /// Parse from string (case-insensitive)
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "user" => Some(Self::User),
            "feedback" => Some(Self::Feedback),
            "project" => Some(Self::Project),
            "reference" => Some(Self::Reference),
            _ => None,
        }
    }

    /// File prefix for this type
    pub fn file_prefix(&self) -> &'static str {
        match self {
            Self::User => "user_",
            Self::Feedback => "feedback_",
            Self::Project => "project_",
            Self::Reference => "reference_",
        }
    }

    /// Display name
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Feedback => "feedback",
            Self::Project => "project",
            Self::Reference => "reference",
        }
    }
}

impl std::fmt::Display for MemoryType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Frontmatter structure for memory files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(rename = "type")]
    pub memory_type: MemoryType,
}

/// Represents a parsed memory file
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub frontmatter: MemoryFrontmatter,
    pub content: String,
    pub path: PathBuf,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(frontmatter: MemoryFrontmatter, content: String, path: PathBuf) -> Self {
        Self {
            frontmatter,
            content,
            path,
        }
    }

    /// Get the file name without extension
    pub fn file_stem(&self) -> Option<String> {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(|s| s.to_string())
    }
}

/// Parse frontmatter from YAML string
/// Returns (frontmatter, body_content) if successful
pub fn parse_frontmatter(content: &str) -> Option<(MemoryFrontmatter, String)> {
    let content = content.trim();
    if !content.starts_with("---") {
        return None;
    }

    let end_marker = content[3..].find("---")?;
    let yaml_content = &content[3..end_marker + 3];
    let body_content = content[end_marker + 6..].trim().to_string();

    let frontmatter: MemoryFrontmatter = serde_yaml::from_str(yaml_content).ok()?;

    Some((frontmatter, body_content))
}

/// Parse a complete memory file
pub fn parse_memory_file(path: &Path) -> anyhow::Result<MemoryEntry> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read memory file: {}", path.display()))?;

    let (frontmatter, body) = parse_frontmatter(&content)
        .ok_or_else(|| anyhow::anyhow!("Invalid frontmatter format in: {}", path.display()))?;

    Ok(MemoryEntry {
        frontmatter,
        content: body,
        path: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_type_from_str() {
        assert_eq!(MemoryType::from_str("user"), Some(MemoryType::User));
        assert_eq!(MemoryType::from_str("USER"), Some(MemoryType::User));
        assert_eq!(MemoryType::from_str("feedback"), Some(MemoryType::Feedback));
        assert_eq!(MemoryType::from_str("project"), Some(MemoryType::Project));
        assert_eq!(MemoryType::from_str("reference"), Some(MemoryType::Reference));
        assert_eq!(MemoryType::from_str("unknown"), None);
    }

    #[test]
    fn test_memory_type_file_prefix() {
        assert_eq!(MemoryType::User.file_prefix(), "user_");
        assert_eq!(MemoryType::Feedback.file_prefix(), "feedback_");
        assert_eq!(MemoryType::Project.file_prefix(), "project_");
        assert_eq!(MemoryType::Reference.file_prefix(), "reference_");
    }

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
name: test-memory
description: A test memory
type: user
---

This is the memory content.
"#;

        let (fm, body) = parse_frontmatter(content).unwrap();
        assert_eq!(fm.name, "test-memory");
        assert_eq!(fm.description, "A test memory");
        assert_eq!(fm.memory_type, MemoryType::User);
        assert_eq!(body, "This is the memory content.");
    }

    #[test]
    fn test_parse_frontmatter_invalid() {
        assert!(parse_frontmatter("not valid frontmatter").is_none());
        assert!(parse_frontmatter("---\nname: test\n---").is_none());
    }
}
