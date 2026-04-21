//! MEMORY.md Index Management
//!
//! Handles parsing and manipulation of MEMORY.md index file.
//! MEMORY.md is an index that points to topic memory files.

use std::fmt;

/// Index entry parsed from MEMORY.md
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub title: String,
    pub file: String,
    pub hook: String,
}

impl IndexEntry {
    /// Parse from a line like: `- [Title](file.md) — hook`
    pub fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if !line.starts_with("- [") {
            return None;
        }

        let after_bracket = line.strip_prefix("- [")?;
        let title_end = after_bracket.find("](")?;
        let title = after_bracket[..title_end].to_string();

        let after_title = &after_bracket[title_end + 2..];
        let file_end = after_title.find(')')?;
        let file = after_title[..file_end].to_string();

        let rest = after_title[file_end + 1..].trim();
        let hook = if let Some(stripped) = rest.strip_prefix("—") {
            stripped.trim().to_string()
        } else if let Some(stripped) = rest.strip_prefix('-') {
            stripped.trim().to_string()
        } else {
            rest.to_string()
        };

        Some(Self { title, file, hook })
    }

    /// Format as a MEMORY.md line
    pub fn to_line(&self) -> String {
        if self.hook.is_empty() {
            format!("- [{}]({})", self.title, self.file)
        } else {
            format!("- [{}]({}) — {}", self.title, self.file, self.hook)
        }
    }
}

/// MEMORY.md index management
#[derive(Debug, Clone, Default)]
pub struct MemoryIndex {
    entries: Vec<IndexEntry>,
}

impl MemoryIndex {
    /// Create a new empty index
    pub fn new() -> Self {
        Self::default()
    }

    /// Load from MEMORY.md content
    pub fn parse(content: &str) -> Self {
        let entries = content
            .lines()
            .filter_map(IndexEntry::parse)
            .collect();
        Self { entries }
    }

    /// Get all entries
    pub fn entries(&self) -> &[IndexEntry] {
        &self.entries
    }

    /// Add or update an entry
    pub fn upsert(&mut self, entry: IndexEntry) {
        if let Some(existing) = self.entries.iter_mut().find(|e| e.file == entry.file) {
            *existing = entry;
        } else {
            self.entries.push(entry);
        }
    }

    /// Remove an entry by file
    pub fn remove(&mut self, file: &str) {
        self.entries.retain(|e| e.file != file);
    }

    /// Check if index is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Serialize to MEMORY.md format
    pub fn to_content(&self) -> String {
        if self.entries.is_empty() {
            return String::new();
        }
        self.entries
            .iter()
            .map(|e| e.to_line())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Find entry by file name
    pub fn find_by_file(&self, file: &str) -> Option<&IndexEntry> {
        self.entries.iter().find(|e| e.file == file)
    }

    /// Filter entries by memory type prefix
    pub fn filter_by_type(&self, type_prefix: &str) -> Vec<&IndexEntry> {
        self.entries
            .iter()
            .filter(|e| e.file.starts_with(type_prefix))
            .collect()
    }
}

impl fmt::Display for MemoryIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_content())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_index_entry() {
        let line = "- [User role](user_role.md) — Data scientist focused on ML";
        let entry = IndexEntry::parse(line).unwrap();
        assert_eq!(entry.title, "User role");
        assert_eq!(entry.file, "user_role.md");
        assert_eq!(entry.hook, "Data scientist focused on ML");
    }

    #[test]
    fn test_parse_index_entry_simple() {
        let line = "- [Test](test.md)";
        let entry = IndexEntry::parse(line).unwrap();
        assert_eq!(entry.title, "Test");
        assert_eq!(entry.file, "test.md");
        assert!(entry.hook.is_empty());
    }

    #[test]
    fn test_index_entry_to_line() {
        let entry = IndexEntry {
            title: "User role".to_string(),
            file: "user_role.md".to_string(),
            hook: "Data scientist".to_string(),
        };
        assert_eq!(
            entry.to_line(),
            "- [User role](user_role.md) — Data scientist"
        );
    }

    #[test]
    fn test_memory_index_parse() {
        let content = "- [User role](user_role.md) — Data scientist\n- [Feedback](feedback.md)";
        let index = MemoryIndex::parse(content);
        assert_eq!(index.len(), 2);
    }

    #[test]
    fn test_memory_index_upsert() {
        let mut index = MemoryIndex::new();
        index.upsert(IndexEntry {
            title: "Test".to_string(),
            file: "test.md".to_string(),
            hook: "Hook".to_string(),
        });
        assert_eq!(index.len(), 1);

        index.upsert(IndexEntry {
            title: "Test Updated".to_string(),
            file: "test.md".to_string(),
            hook: "Updated".to_string(),
        });
        assert_eq!(index.len(), 1);
        assert_eq!(index.entries[0].title, "Test Updated");
    }

    #[test]
    fn test_memory_index_remove() {
        let mut index = MemoryIndex::new();
        index.upsert(IndexEntry {
            title: "Test".to_string(),
            file: "test.md".to_string(),
            hook: "Hook".to_string(),
        });
        index.remove("test.md");
        assert!(index.is_empty());
    }
}
