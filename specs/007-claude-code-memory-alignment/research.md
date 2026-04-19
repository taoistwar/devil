# Research: Claude Code Memory System

**Date**: 2026-04-19
**Feature**: 007-claude-code-memory-alignment

## Research Questions

### RQ1: Memory Type Taxonomy

**Question**: What are the four memory types and their intended use cases?

**Finding**: Claude Code defines four memory types:
- **user**: Information about user's role, goals, responsibilities, knowledge
- **feedback**: Guidance from user on what to avoid or continue doing
- **project**: Ongoing work, goals, initiatives, bugs, incidents not derivable from code
- **reference**: Pointers to external systems (Linear, Slack, Grafana)

**Decision**: Implement all four types following reference exactly.
**Rationale**: Eval-validated taxonomy that structures memory appropriately.

### RQ2: Memory Directory Path Resolution

**Question**: How to determine the memory directory path?

**Finding**: Claude Code uses resolution order:
1. `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` env var (full override)
2. `autoMemoryDirectory` in settings.json (user/policy/local only, not project)
3. `<memoryBase>/projects/<sanitized-git-root>/memory/`

**Decision**: Follow same resolution order with env var override first.
**Rationale**: Matches Claude Code behavior, respects security (projectSettings excluded).

### RQ3: MEMORY.md Truncation Strategy

**Question**: How to handle large MEMORY.md files?

**Finding**: Two-stage truncation:
1. Line truncation: First 200 lines (MAX_ENTRYPOINT_LINES)
2. Byte truncation: 25KB max (MAX_ENTRYPOINT_BYTES), cut at last newline

**Decision**: Implement both truncation strategies with warning messages.
**Rationale**: Prevents token overflow while preserving readability.

### RQ4: Frontmatter Parsing

**Question**: How to parse and validate frontmatter?

**Finding**: Claude Code uses YAML frontmatter with required fields:
- `name`: Memory identifier
- `description`: One-line description for relevance
- `type`: One of the four types

**Decision**: Use `serde_yaml` for parsing with validation.
**Rationale**: YAML is standard format, serde_yaml is idiomatic Rust.

### RQ5: Memory Guidance Prompts

**Question**: What guidance text should be included?

**Finding**: Claude Code includes:
- Types of memory section (per-type guidance)
- What NOT to save section
- When to access memories section
- Before recommending from memory section (verification guidance)
- How to save memories section (two-step process)

**Decision**: Include all guidance text from reference.
**Rationale**: Eval-validated prompting produces best results.

## Alternatives Considered

### Alternative 1: Simple Key-Value Store

**Description**: Store memories as simple key-value pairs in JSON
**Rejected because**: Doesn't support rich metadata, frontmatter, or type taxonomy
**Decision**: Use file-based frontmatter format

### Alternative 2: Vector Embedding Retrieval

**Description**: Use embeddings for semantic memory retrieval
**Rejected because**: Over-engineering for initial implementation, requires external service
**Decision**: File-based with manual search (grep)

### Alternative 3: Database Storage

**Description**: SQLite or PostgreSQL for memory storage
**Rejected because**: Adds deployment complexity, Claude Code uses files
**Decision**: File-based for simplicity and parity

## Dependencies

- `serde_yaml` for frontmatter parsing
- `tokio::fs` for async file operations
- `anyhow` for error handling with context

## Risks

| Risk | Mitigation |
|------|------------|
| Large memory files slow to read | Async I/O, consider caching |
| Frontmatter parsing errors | Graceful degradation, log warnings |
| Path security (traversal) | Validate and sanitize paths |