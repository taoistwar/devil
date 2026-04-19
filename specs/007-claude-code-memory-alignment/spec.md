# Feature Specification: Claude Code Memory System Alignment

**Feature Branch**: `007-claude-code-memory-alignment`  
**Created**: 2026-04-19  
**Status**: Draft  
**Input**: User description: "读取下claude code 的 references/claude-code/src/memory.ts，理解里面的功能特性，让后让当前项目对齐它"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Persistent Memory Storage (Priority: P1)

As a user, I want the agent to remember information about me, my preferences, and my project across conversations so that I don't need to repeat myself.

**Why this priority**: This is the core value proposition of the memory system - eliminating repetition and building context over time.

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

### Edge Cases

- What happens when MEMORY.md is empty or doesn't exist?
- How does the system handle duplicate memory entries?
- What happens when a memory conflicts with current code state?
- How does the system handle very long memory entries (>200 chars per line)?
- What happens when memory files are corrupted or have invalid frontmatter?
- How does the system behave in bare/simple mode?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support four memory types: user, feedback, project, reference
- **FR-002**: System MUST organize memories in a memory directory with MEMORY.md as the index
- **FR-003**: Memory files MUST use frontmatter format with name, description, and type fields
- **FR-004**: System MUST truncate MEMORY.md at 200 lines and 25KB to prevent token overflow
- **FR-005**: System MUST provide guidance on what NOT to save in memory
- **FR-006**: System MUST verify memory accuracy against current state before acting on old information
- **FR-007**: System MUST allow users to explicitly ask to remember or forget information
- **FR-008**: System MUST ignore memories when user explicitly says to do so
- **FR-009**: Memory directory path MUST follow pattern: ~/.claude/projects/<project>/memory/
- **FR-010**: System MUST discover and load CLAUDE.md files (existing functionality preserved)
- **FR-011**: System MUST support environment variable CLAUDE_CODE_DISABLE_AUTO_MEMORY to disable memory
- **FR-012**: System MUST log memory directory file counts for telemetry

### Key Entities *(include if feature involves data)*

- **MemoryFile**: Represents a memory file with path, content, and metadata
- **MemoryType**: Enum of 4 types (user, feedback, project, reference)
- **MemoryEntry**: Frontmatter structure with name, description, type
- **MemoryDir**: Directory structure containing MEMORY.md index and topic files
- **EntrypointTruncation**: Truncation result with content, line count, byte count, and truncation flags

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Memories persist across sessions and can be retrieved in new conversations
- **SC-002**: MEMORY.md index remains under 200 lines and 25KB through truncation
- **SC-003**: Each memory type (user, feedback, project, reference) can be saved and retrieved independently
- **SC-004**: Memory guidance is included in system prompt when auto memory is enabled
- **SC-005**: Agent respects "ignore memory" instructions when explicitly requested
- **SC-006**: Memory files are stored in the correct directory structure
- **SC-007**: Frontmatter format is validated and parseable

## Assumptions

- Users have ~/.claude directory available for memory storage
- Memory system is disabled by default in bare/simple mode
- Team memory features are out of scope for initial implementation (feature-gated in Claude Code)
- KAIROS daily-log mode is out of scope for initial implementation
- The project will leverage existing Rust async file I/O patterns
- Memory frontmatter will use YAML format for compatibility