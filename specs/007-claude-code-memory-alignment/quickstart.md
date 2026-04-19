# Quickstart: Claude Code Memory System

**Feature**: 007-claude-code-memory-alignment
**Last Updated**: 2026-04-19

## Overview

The memory system provides persistent, file-based memory storage organized by type. Memories persist across conversations and are loaded into the agent's context automatically.

## Memory Types

| Type | When to Save | Example |
|------|-------------|---------|
| `user` | User role, goals, preferences | "user is a data scientist focused on ML" |
| `feedback` | Corrections, confirmations | "don't mock the database in tests" |
| `project` | Deadlines, decisions, context | "merge freeze starts 2026-03-05" |
| `reference` | External system pointers | "pipeline bugs tracked in Linear INGEST" |

## Memory Directory

```
~/.claude/projects/<project>/memory/
├── MEMORY.md           # Index file (entrypoint)
├── user_role.md        # User memories
├── feedback_testing.md # Feedback memories
└── reference_linear.md # Reference memories
```

## Environment Variables

| Variable | Effect |
|----------|--------|
| `CLAUDE_CODE_DISABLE_AUTO_MEMORY=1` | Disable memory system |
| `CLAUDE_CODE_SIMPLE` or `--bare` | Disable memory (bare mode) |
| `CLAUDE_COWORK_MEMORY_PATH_OVERRIDE` | Override memory directory path |

## Usage

### Saving a Memory

When the user provides guidance or information worth remembering:

1. Write the memory to a topic file with frontmatter:
```markdown
---
name: user-role
description: Data scientist focused on ML/observability
type: user
---

The user is a data scientist investigating logging patterns in the codebase.
```

2. Add an index entry to MEMORY.md:
```markdown
- [User role](user_role.md) — Data scientist focused on ML/observability
```

### Accessing Memories

Memories are automatically loaded into context when:
- Auto memory is enabled (not in bare mode)
- MEMORY.md exists in the memory directory

### Forgetting a Memory

When user asks to forget something:
1. Find the memory file
2. Delete the file
3. Remove the index entry from MEMORY.md

## What NOT to Save

Do not save in memory:
- Code patterns, conventions, architecture (read from code)
- Git history (use `git log`)
- Debugging solutions (in the code/commit)
- Information in CLAUDE.md files
- Ephemeral task details

## Truncation

MEMORY.md is truncated at:
- **200 lines** (line count)
- **25KB** (byte count, at last newline)

If truncated, a warning message is appended.

## CLI Command

```bash
# View memory status
/memory

# Future commands (not yet implemented)
/memory save <type> <name> <content>
/memory list [--type <type>]
/memory forget <name>
/memory search <query>
```