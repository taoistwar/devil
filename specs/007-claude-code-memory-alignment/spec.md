# Feature Specification: Claude Code Memory System Alignment

**Feature Branch**: `007-claude-code-memory-alignment`  
**Created**: 2026-04-19  
**Status**: Draft  
**Input**: User description: "读取下claude code 的 references/claude-code/src/memory.ts，理解里面的功能特性，让后让当前项目对齐它"

## 1. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Memory System Architecture                          │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                        Memory Storage Layer                          │   │
│  │                                                                      │   │
│  │   ~/.claude/projects/<project>/memory/                              │   │
│  │   ├── MEMORY.md                    # Index file (entry point)       │   │
│  │   ├── user/                        # User type memories              │   │
│  │   │   ├── preferences.md                                         │   │
│  │   │   └── background.md                                          │   │
│  │   ├── feedback/                     # Feedback type memories          │   │
│  │   │   └── corrections.md                                          │   │
│  │   ├── project/                      # Project type memories          │   │
│  │   │   ├── deadlines.md                                            │   │
│  │   │   └── context.md                                              │   │
│  │   └── reference/                    # Reference type memories        │   │
│  │       └── external-systems.md                                     │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                      │                                        │
│  ┌───────────────────────────────────▼───────────────────────────────────┐   │
│  │                        Memory Types (4 types)                         │   │
│  │                                                                      │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌───────────┐ │   │
│  │   │    user     │  │   feedback   │  │   project   │  │ reference │ │   │
│  │   │  用户信息    │  │   用户纠正    │  │  项目信息    │  │ 外部引用  │ │   │
│  │   │  偏好设置    │  │   反馈指导    │  │  截止日期    │  │ Linear   │ │   │
│  │   │  角色背景    │  │   纠正记录    │  │  项目状态    │  │ Slack    │ │   │
│  │   └─────────────┘  └─────────────┘  └─────────────┘  └───────────┘ │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                      │                                        │
│  ┌───────────────────────────────────▼───────────────────────────────────┐   │
│  │                    Memory Lifecycle Management                        │   │
│  │                                                                      │   │
│  │   Short-term Memory          │          Long-term Memory             │   │
│  │   ───────────────────────── │          ─────────────────────────   │   │
│  │   • Current session only    │          • Persists across sessions  │   │
│  │   • In-memory cache          │          • Stored in filesystem      │   │
│  │   • Auto-expire after 7d    │          • TTL: 90 days default       │   │
│  │                              │          • Refresh on access          │   │
│  │                                                                      │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 2. Storage Capacity Definition

### 2.1 Storage Limits

| Metric | Limit | Behavior When Exceeded |
|--------|-------|------------------------|
| **MEMORY.md Index** | 200 lines | Truncate with warning message |
| **MEMORY.md Size** | 25 KB | Byte-truncate at last newline before limit |
| **Per Memory File** | 50 KB | Reject save, return error |
| **Total Memory Directory** | 10 MB | Reject new saves, prompt cleanup |
| **Single Memory Entry** | 4096 chars | Reject save, return error |

### 2.2 Capacity Enforcement

```rust
pub struct MemoryCapacity {
    pub max_index_lines: usize = 200,
    pub max_index_bytes: usize = 25 * 1024,
    pub max_file_bytes: usize = 50 * 1024,
    pub max_total_bytes: usize = 10 * 1024 * 1024,
    pub max_entry_chars: usize = 4096,
}
```

### 2.3 Truncation Behavior

```rust
pub struct EntrypointTruncation {
    pub content: String,           // Truncated content
    pub line_count: usize,         // Original line count
    pub byte_count: usize,          // Original byte count
    pub was_truncated: bool,        // True if truncation occurred
    pub truncation_type: Option<TruncationType>,
}

pub enum TruncationType {
    LineLimit,    // Exceeded 200 lines
    ByteLimit,    // Exceeded 25KB
}
```

## 3. Memory Types Specification

### 3.1 Type Definitions

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MemoryType {
    /// User information, preferences, and background
    User,
    /// User corrections and feedback
    Feedback,
    /// Project-specific information and deadlines
    Project,
    /// External system references (Linear, Slack, Grafana)
    Reference,
}

impl MemoryType {
    pub fn directory(&self) -> &'static str {
        match self {
            MemoryType::User => "user",
            MemoryType::Feedback => "feedback",
            MemoryType::Project => "project",
            MemoryType::Reference => "reference",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MemoryType::User => "User information, preferences, and background",
            MemoryType::Feedback => "User corrections and feedback to the agent",
            MemoryType::Project => "Project-specific information and deadlines",
            MemoryType::Reference => "External system references (Linear, Slack, Grafana)",
        }
    }
}
```

### 3.2 Memory Entry Frontmatter Format

All memory files MUST use YAML frontmatter with the following structure:

```yaml
---
name: <unique-memory-name>
type: <user|feedback|project|reference>
created: <YYYY-MM-DDTHH:MM:SSZ>
expires: <YYYY-MM-DDTHH:MM:SSZ>  # Optional, for long-term memory
tags: [<tag1>, <tag2>]           # Optional
project: <project-name>          # Optional, for project/reference types
---

# Memory content goes here
# This can include multiple paragraphs, lists, etc.
```

#### Frontmatter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique identifier for this memory |
| `type` | enum | Yes | One of: user, feedback, project, reference |
| `created` | ISO8601 | Yes | Creation timestamp |
| `expires` | ISO8601 | No | Expiration timestamp (long-term memory only) |
| `tags` | string[] | No | Optional tags for categorization |
| `project` | string | No | Associated project name |

### 3.3 Memory File Naming Convention

```
<type>/<unique-name>.md
```

Examples:
- `user/preferences.md`
- `feedback/correction-2026-04-20.md`
- `project/deadlines.md`
- `reference/linear-ingest.md`

## 4. Memory Lifecycle Management

### 4.1 Long-term vs Short-term Memory

```rust
pub enum MemoryScope {
    /// Persists across sessions, stored in filesystem
    LongTerm {
        ttl_days: u32,           // Time-to-live in days
        refresh_on_access: bool, // Extend TTL on access
    },
    /// Current session only, stored in-memory
    ShortTerm {
        max_age_hours: u32,      // Auto-expire after N hours
    },
}

impl Default for MemoryScope {
    fn default() -> Self {
        MemoryScope::LongTerm {
            ttl_days: 90,
            refresh_on_access: true,
        }
    }
}
```

### 4.2 Default TTL by Memory Type

| Memory Type | Default TTL | Refresh on Access |
|-------------|-------------|-------------------|
| user | 180 days | Yes |
| feedback | 90 days | Yes |
| project | 30 days | Yes |
| reference | 14 days | No |

### 4.3 Expiration Policy

```rust
pub struct MemoryExpirationPolicy {
    /// Default TTL in days for each memory type
    pub default_ttl: HashMap<MemoryType, u32>,
    /// Whether to extend TTL when memory is accessed
    pub refresh_on_access: bool,
    /// Grace period after expiration before deletion (days)
    pub grace_period_days: u32,
    /// Background cleanup interval (hours)
    pub cleanup_interval_hours: u32,
}

impl Default for MemoryExpirationPolicy {
    fn default() -> Self {
        let mut default_ttl = HashMap::new();
        default_ttl.insert(MemoryType::User, 180);
        default_ttl.insert(MemoryType::Feedback, 90);
        default_ttl.insert(MemoryType::Project, 30);
        default_ttl.insert(MemoryType::Reference, 14);

        MemoryExpirationPolicy {
            default_ttl,
            refresh_on_access: true,
            grace_period_days: 7,
            cleanup_interval_hours: 24,
        }
    }
}
```

### 4.4 Memory Refresh Behavior

- **On Read**: If `refresh_on_access` is true, update the `expires` field
- **On Write**: Always update the `expires` field based on TTL
- **Expired Memory**: Soft delete during grace period, hard delete after

## 5. MEMORY.md Index Management

### 5.1 Index File Structure

```markdown
# Memory Index

## Summary
Total Memories: <count> | Last Updated: <YYYY-MM-DD>

## User Memories
- [user/preferences](user/preferences.md) - User preferences and settings
- [user/background](user/background.md) - User background information

## Feedback Memories
- [feedback/corrections](feedback/corrections.md) - User corrections and feedback

## Project Memories
- [project/deadlines](project/deadlines.md) - Project deadlines and milestones
- [project/context](project/context.md) - Current project context

## Reference Memories
- [reference/linear](reference/linear.md) - Linear project references
- [reference/slack](reference/slack.md) - Slack channel references

<!-- TRUNCATED -->
Last memory load truncated at line 200 / 25KB
```

### 5.2 Index Entry Format

Each entry in MEMORY.md follows this format:

```markdown
- [<type>/<filename>](<type>/<filename>.md) - <one-line-description>
```

### 5.3 Index Loading Rules

1. **On Session Start**: Load MEMORY.md into context (up to 200 lines / 25KB)
2. **On Memory Access**: Load corresponding topic file content
3. **On Truncation**: Include warning message at truncation point

```rust
pub struct MemoryIndex {
    pub path: PathBuf,
    pub entries: Vec<MemoryIndexEntry>,
    pub total_count: usize,
    pub last_updated: DateTime<Utc>,
    pub was_truncated: bool,
    pub truncation_info: Option<TruncationInfo>,
}

pub struct MemoryIndexEntry {
    pub memory_type: MemoryType,
    pub filename: String,
    pub relative_path: PathBuf,
    pub one_line_description: String,
}
```

## 6. Memory Classification Guidance

### 6.1 What TO Save (Should Save)

| Category | Examples |
|----------|----------|
| **User Preferences** | Response style (concise/detailed), communication preferences, tool preferences |
| **User Background** | Role, expertise, industry, current projects |
| **Project Context** | Deadlines, milestones, architecture decisions, team structure |
| **External References** | Linear project IDs, Slack channels, Grafana dashboards |
| **Feedback History** | Corrected mistakes, preferred approaches, avoid suggestions |

### 6.2 What NOT to Save (Should NOT Save)

| Category | Reason | Alternative |
|----------|--------|-------------|
| **Code Patterns** | Derivable from code analysis | N/A - don't save |
| **Git History** | Available via `git log`, `git blame` | Use git commands |
| **File Contents** | Can be read from filesystem | Use file tools |
| **CLAUDE.md Content** | Already loaded by context system | N/A - don't duplicate |
| **Tool Results** | Already in context | N/A - don't save |
| **Conversation History** | Available in session context | N/A - don't save |
| **Obvious Facts** | Public knowledge, easily verifiable | N/A - don't save |
| **Temporary State** | Session-specific, not persistent | Use short-term memory |
| **Credentials/Secrets** | Security risk | Use proper secret management |

### 6.3 Memory Save Decision Flow

```
Should I save this as a memory?

1. Is this information derivable from code/git/docs?
   → NO: Continue to step 2
   → YES: DO NOT SAVE, explain why

2. Is this already stored in CLAUDE.md or context?
   → NO: Continue to step 3
   → YES: DO NOT SAVE, reference existing location

3. Is this information persistent across sessions?
   → NO: Consider short-term memory only
   → YES: Continue to step 4

4. Does this fit one of the 4 memory types?
   → YES: Save with appropriate type and TTL
   → NO: DO NOT SAVE, explain why
```

## 7. User Scenarios & Testing

### User Story 1 - Persistent Memory Storage (Priority: P1)

As a user, I want the agent to remember information about me, my preferences, and my project across conversations so that I don't need to repeat myself.

**Why this priority**: Core value proposition - eliminating repetition and building context over time.

**Independent Test**: When a user saves a memory and then starts a new conversation, the agent can recall that information.

**Acceptance Scenarios**:

1. **Given** a user has saved a memory about their role (e.g., "data scientist"), **When** they start a new conversation, **Then** the agent should know their role and tailor responses accordingly.

2. **Given** a user has saved feedback about preferring concise responses, **When** they start a new conversation, **Then** the agent should not summarize responses at the end.

3. **Given** a user has saved a project memory about a deadline, **When** they ask about project status, **Then** the agent should reference the saved deadline.

---

### User Story 2 - Typed Memory Organization (Priority: P1)

As a user, I want memories to be organized by type (user, feedback, project, reference) so that the agent can reason about when to use different types of knowledge.

**Why this priority**: The taxonomy ensures memories are used appropriately and not confused with derivable information.

**Independent Test**: Each memory type is stored separately and the agent accesses appropriate types based on context.

**Acceptance Scenarios**:

1. **Given** a user saves a memory with type "user", **When** the agent needs to understand the user, **Then** it should access user-type memories.

2. **Given** a user saves a memory with type "feedback", **When** the user corrects the agent, **Then** the agent should save it as feedback type.

3. **Given** a user saves a memory with type "reference", **When** the user mentions Linear or Slack, **Then** the agent should recall the reference memory.

---

### User Story 3 - MEMORY.md Index Management (Priority: P2)

As a user, I want an index file (MEMORY.md) that points to topic files so that the agent can quickly find relevant memories without loading everything.

**Why this priority**: MEMORY.md serves as the entry point that gets loaded into context, keeping token usage manageable.

**Independent Test**: MEMORY.md contains pointers to topic files, and topic files contain the actual memory content.

**Acceptance Scenarios**:

1. **Given** a user saves a new memory, **When** the agent writes the topic file and updates MEMORY.md index, **Then** the index should contain a one-line pointer to the topic file.

2. **Given** MEMORY.md exceeds 200 lines, **When** memories are loaded, **Then** the content should be truncated with a warning message.

3. **Given** MEMORY.md exceeds 25KB, **When** memories are loaded, **Then** the content should be byte-truncated at the last newline before the limit.

---

### User Story 4 - Memory Exclusion Guidance (Priority: P2)

As a user, I want the agent to understand what NOT to save in memory so that memories stay relevant and don't duplicate derivable information.

**Why this priority**: Prevents memory pollution with information that can be obtained from code, git, or documentation.

**Independent Test**: Agent should not save code patterns, git history, or CLAUDE.md content as memories.

**Acceptance Scenarios**:

1. **Given** a user asks to save a code pattern, **When** the agent evaluates the save request, **Then** it should explain that code patterns are derivable and not save them.

2. **Given** a user asks to save git history, **When** the agent evaluates the save request, **Then** it should explain that git history is available via git commands.

---

### User Story 5 - External System References (Priority: P3)

As a user, I want the agent to remember pointers to external systems (Linear, Slack, Grafana) so it knows where to find information outside the project.

**Why this priority**: External systems change frequently; storing pointers keeps memories accurate without needing constant updates.

**Independent Test**: When a user mentions an external system, the agent can save and later recall the pointer.

**Acceptance Scenarios**:

1. **Given** a user says "check Linear project INGEST for pipeline bugs", **When** the agent saves the memory, **Then** it should be saved as a reference type memory.

2. **Given** a reference memory points to Grafana dashboard, **When** the user asks about oncall dashboards, **Then** the agent should recall the pointer.

---

### User Story 6 - Memory Expiration & Cleanup (Priority: P2)

As a user, I want old memories to expire automatically so that the memory system stays relevant and doesn't accumulate stale information.

**Why this priority**: Prevents outdated information from polluting context and wasting storage.

**Independent Test**: Memories expire based on their TTL and are cleaned up during maintenance.

**Acceptance Scenarios**:

1. **Given** a reference memory has expired (TTL exceeded), **When** the grace period has passed, **Then** the memory should be automatically deleted.

2. **Given** a user accesses their preferences memory, **When** the memory has `refresh_on_access` enabled, **Then** the expiration date should be extended.

3. **Given** memory storage approaches the 10MB limit, **When** a user tries to save new memory, **Then** the system should prompt cleanup of expired memories.

---

### Edge Cases

- What happens when MEMORY.md is empty or doesn't exist?
  → Create empty index with header only
- How does the system handle duplicate memory entries?
  → Deduplicate by `name` field, update existing entry
- What happens when a memory conflicts with current code state?
  → Always verify against current state before acting on old information
- What happens when memory files are corrupted or have invalid frontmatter?
  → Log error, skip file, notify user via warning
- How does the system behave in bare/simple mode?
  → Memory system is disabled by default, controlled by `CLAUDE_CODE_DISABLE_AUTO_MEMORY`
- What happens when a memory entry exceeds 4096 characters?
  → Reject save with error, suggest splitting into multiple entries

## 8. Functional Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| **FR-001** | System MUST support four memory types: user, feedback, project, reference | P1 |
| **FR-002** | System MUST organize memories in a memory directory with MEMORY.md as the index | P1 |
| **FR-003** | Memory files MUST use YAML frontmatter format with name, type, created, and optional expires fields | P1 |
| **FR-004** | System MUST truncate MEMORY.md at 200 lines and 25KB to prevent token overflow | P1 |
| **FR-005** | System MUST provide guidance on what NOT to save in memory | P1 |
| **FR-006** | System MUST verify memory accuracy against current state before acting on old information | P1 |
| **FR-007** | System MUST allow users to explicitly ask to remember or forget information | P1 |
| **FR-008** | System MUST ignore memories when user explicitly says to do so | P1 |
| **FR-009** | Memory directory path MUST follow pattern: `~/.claude/projects/<project>/memory/` | P1 |
| **FR-010** | System MUST discover and load CLAUDE.md files (existing functionality preserved) | P1 |
| **FR-011** | System MUST support environment variable `CLAUDE_CODE_DISABLE_AUTO_MEMORY` to disable memory | P1 |
| **FR-012** | System MUST log memory directory file counts for telemetry | P2 |
| **FR-013** | System MUST enforce storage capacity limits (MEMORY.md 200 lines/25KB, per-file 50KB, total 10MB) | P1 |
| **FR-014** | System MUST implement TTL-based expiration with configurable per-type defaults | P2 |
| **FR-015** | System MUST support memory refresh on access for long-term memories | P2 |
| **FR-016** | System MUST implement grace period before permanent deletion of expired memories | P2 |

## 9. Key Entities

| Entity | Description |
|--------|-------------|
| **MemoryFile** | Represents a memory file with path, content, and metadata |
| **MemoryType** | Enum of 4 types (user, feedback, project, reference) |
| **MemoryEntry** | Frontmatter structure with name, type, created, expires, tags, project |
| **MemoryDir** | Directory structure containing MEMORY.md index and typed subdirectories |
| **MemoryIndex** | MEMORY.md entry point with pointers to topic files |
| **MemoryScope** | Enum distinguishing long-term vs short-term memory |
| **MemoryCapacity** | Storage limits configuration |
| **MemoryExpirationPolicy** | TTL and cleanup configuration |
| **EntrypointTruncation** | Truncation result with content, line count, byte count, and truncation flags |

## 10. Error Handling

Following SPEC_DEPENDENCIES.md error handling规范:

```rust
pub enum MemoryError {
    /// User input errors (E1xxx)
    InvalidInput {
        code: ErrorCode,
        message: String,
        field: Option<String>,
    },
    /// Resource errors (E3xxx)
    Resource {
        code: ErrorCode,
        message: String,
        resource_type: String,
    },
    /// Internal errors (E5xxx)
    Internal {
        code: ErrorCode,
        message: String,
        stack_trace: Option<String>,
    },
}

impl From<std::io::Error> for MemoryError {
    fn from(err: std::io::Error) -> Self {
        MemoryError::Internal {
            code: ErrorCode::InternalError,
            message: err.to_string(),
            stack_trace: None,
        }
    }
}
```

### Error Codes

| Code | Name | Description |
|------|------|-------------|
| E3001 | ResourceNotFound | Memory file or directory not found |
| E3002 | ResourceConflict | Duplicate memory name |
| E3003 | ResourceExhausted | Storage capacity exceeded |
| E1003 | InvalidFormat | Invalid frontmatter or content format |

## 11. Success Criteria

### Measurable Outcomes

| ID | Criterion | Metric |
|----|-----------|--------|
| **SC-001** | Memories persist across sessions and can be retrieved in new conversations | Session recovery test passes |
| **SC-002** | MEMORY.md index remains under 200 lines and 25KB through truncation | Index stays within limits |
| **SC-003** | Each memory type (user, feedback, project, reference) can be saved and retrieved independently | Type isolation verified |
| **SC-004** | Memory guidance is included in system prompt when auto memory is enabled | Guidance text present |
| **SC-005** | Agent respects "ignore memory" instructions when explicitly requested | Ignore flag honored |
| **SC-006** | Memory files are stored in the correct directory structure | Path pattern verified |
| **SC-007** | Frontmatter format is validated and parseable | YAML parsing succeeds |
| **SC-008** | Expired memories are automatically cleaned up after grace period | Cleanup test passes |
| **SC-009** | Storage capacity is enforced (total 10MB, per-file 50KB, entry 4096 chars) | Capacity tests pass |

## 12. Assumptions

- Users have `~/.claude` directory available for memory storage
- Memory system is disabled by default in bare/simple mode
- Team memory features are out of scope for initial implementation
- KAIROS daily-log mode is out of scope for initial implementation
- The project will leverage existing Rust async file I/O patterns
- Memory frontmatter will use YAML format for compatibility

## 13. Dependencies

| Dependency | Type | Description |
|------------|------|-------------|
| spec-006 | Weak | Context injection may include memory content |
| spec-004 | Weak | Permission system may control memory access |
| serde_yaml | Required | YAML frontmatter parsing |
| tokio | Required | Async file I/O |
| chrono | Required | ISO8601 timestamp handling |

(End of file - total lines: ~450)
