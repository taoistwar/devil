# Data Model: Claude Code Memory System

**Date**: 2026-04-19
**Feature**: 007-claude-code-memory-alignment

## Entity Definitions

### MemoryType

```rust
/// Four memory types aligned with Claude Code's taxonomy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    User,      // User role, goals, preferences
    Feedback,  // What to avoid or continue doing
    Project,   // Work context, deadlines, decisions
    Reference, // External system pointers
}

impl MemoryType {
    /// Parse from string (case-insensitive)
    pub fn from_str(s: &str) -> Option<Self>;
    
    /// File prefix for this type
    pub fn file_prefix(&self) -> &'static str;
}
```

### MemoryFrontmatter

```rust
/// Frontmatter structure for memory files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFrontmatter {
    pub name: String,
    pub description: String,
    pub #[serde(rename = "type")] 
    memory_type: MemoryType,
}

/// Parse frontmatter from YAML string
pub fn parse_frontmatter(content: &str) -> Option<(MemoryFrontmatter, &str)>;
```

### MemoryEntry

```rust
/// Represents a parsed memory file
#[derive(Debug, Clone)]
pub struct MemoryEntry {
    pub frontmatter: MemoryFrontmatter,
    pub content: String,
    pub path: PathBuf,
}

/// Parse a complete memory file
pub fn parse_memory_file(path: &Path) -> anyhow::Result<MemoryEntry>;
```

### MemoryDir

```rust
/// Memory directory configuration
#[derive(Debug, Clone)]
pub struct MemoryDir {
    path: PathBuf,
}

impl MemoryDir {
    /// Create from environment/settings resolution
    pub fn resolve() -> Self;
    
    /// Get MEMORY.md index path
    pub fn index_path(&self) -> PathBuf;
    
    /// Get path for a new memory file
    pub fn memory_path(&self, memory_type: MemoryType, name: &str) -> PathBuf;
    
    /// List all memory files
    pub async fn list_memories(&self) -> anyhow::Result<Vec<MemoryEntry>>;
}
```

### EntrypointTruncation

```rust
/// Result of truncating MEMORY.md content
#[derive(Debug, Clone)]
pub struct EntrypointTruncation {
    pub content: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub was_line_truncated: bool,
    pub was_byte_truncated: bool,
}

/// Constants aligned with Claude Code
pub const MAX_ENTRYPOINT_LINES: usize = 200;
pub const MAX_ENTRYPOINT_BYTES: usize = 25_000;

/// Truncate MEMORY.md content to line and byte limits
pub fn truncate_entrypoint(raw: &str) -> EntrypointTruncation;
```

### MemoryIndex

```rust
/// MEMORY.md index entry
#[derive(Debug, Clone)]
pub struct IndexEntry {
    pub title: String,
    pub file: String,
    pub hook: String,
}

/// Parse or generate MEMORY.md index
pub struct MemoryIndex {
    entries: Vec<IndexEntry>,
}

impl MemoryIndex {
    /// Load from MEMORY.md content
    pub fn parse(content: &str) -> Self;
    
    /// Add or update an entry
    pub fn upsert(&mut self, entry: IndexEntry);
    
    /// Remove an entry by file
    pub fn remove(&mut self, file: &str);
    
    /// Serialize to MEMORY.md format
    pub fn to_string(&self) -> String;
}
```

### MemoryGuidance

```rust
/// Static guidance text from Claude Code memory.ts
pub struct MemoryGuidance;

impl MemoryGuidance {
    /// Build complete memory prompt (without MEMORY.md content)
    pub fn build_prompt(memory_dir: &str) -> Vec<&'static str>;
    
    /// Types section (individual mode)
    pub const TYPES_SECTION_INDIVIDUAL: &'static [&'static str];
    
    /// What NOT to save section
    pub const WHAT_NOT_TO_SAVE: &'static [&'static str];
    
    /// When to access section
    pub const WHEN_TO_ACCESS: &'static [&'static str];
    
    /// Trusting recall section
    pub const TRUSTING_RECALL: &'static [&'static str];
}
```

## State Transitions

### Memory Lifecycle

```
[User Request] 
     │
     ▼
[Evaluate Type] ──► [Save Memory]
     │                   │
     │                   ├── Write topic file (name.md)
     │                   └── Update MEMORY.md index
     │
[Recall Memory] ◄── [Search Index] ◄─── [Query]
     │
     ▼
[Verify Against Current State]
     │
     ▼
[Use Memory / Update / Discard]
```

### Memory Operations

| Operation | Input | Output | Side Effects |
|-----------|-------|--------|--------------|
| Save | MemoryEntry | Result | Creates file, updates index |
| Recall | Query | Vec<MemoryEntry> | None (read-only) |
| Forget | Memory name | Result | Removes file, updates index |
| List | Filter (type) | Vec<MemoryEntry> | None |

## Validation Rules

1. **Frontmatter required fields**: name, description, type
2. **Type must be valid**: One of four MemoryType variants
3. **Index entries**: One line, <150 chars, format: `- [Title](file.md) — hook`
4. **File naming**: `<type>_<name>.md` with alphanumeric + underscore

## Error Handling

| Error | Handling |
|-------|----------|
| Invalid frontmatter | Log warning, skip file |
| Missing type | Log warning, default to "user" |
| Duplicate memory | Overwrite existing |
| Index parse error | Regenerate from files |
| Path traversal attempt | Reject with security error |