# Data Model: Claude Code Context Alignment

## Overview

This document describes the data structures for context injection aligned with Claude Code's context.ts.

## Entities

### GitStatus

Git repository status information.

| Field | Type | Description |
|-------|------|-------------|
| `branch` | `String` | Current git branch name |
| `main_branch` | `String` | Default/main branch name |
| `user_name` | `Option<String>` | Git user name from config |
| `status` | `String` | Short git status output |
| `recent_commits` | `String` | Last 5 commits in oneline format |

### MemoryFile

A discovered CLAUDE.md file.

| Field | Type | Description |
|-------|------|-------------|
| `path` | `PathBuf` | Absolute path to the file |
| `content` | `String` | File contents |

### SystemContext

System-level context injected before conversation.

| Field | Type | Description |
|-------|------|-------------|
| `git_status` | `Option<GitStatus>` | Git status if in git repo |
| `cache_breaker` | `Option<String>` | Cache breaker injection string |

### UserContext

User-level context injected before conversation.

| Field | Type | Description |
|-------|------|-------------|
| `memory_files` | `Option<String>` | Combined CLAUDE.md contents |
| `current_date` | `String` | ISO 8601 formatted date |

### ContextProviders

Combined context providers.

| Field | Type | Description |
|-------|------|-------------|
| `system` | `SystemContextProvider` | System context collector |
| `user` | `UserContextProvider` | User context collector |

## State Transitions

### Git Status Collection

```
NotGitRepo → NoStatus (skip collection)
GitRepo → Collecting → Collected(GitStatus)
GitRepo → Collecting → Error (log and return None)
```

### Memory Files Discovery

```
Scanning → Found(Vec<MemoryFile>) | NotFound
Disabled → Skipped (return None)
```

## Validation Rules

1. **GitStatus**:
   - `branch`: Non-empty if git repo
   - `status`: Always non-null (empty string if clean)
   - `recent_commits`: Always non-null (empty string if no commits)

2. **MemoryFile**:
   - `path`: Must exist and be readable
   - `content`: May be empty

3. **Context Providers**:
   - All providers are memoized (single collection per session)
   - Cache can be cleared explicitly

## Relationships

```
ContextProviders
├── SystemContextProvider
│   └── uses: GitStatusCollector
│   └── uses: SystemPromptInjection (static)
└── UserContextProvider
    └── uses: MemoryFilesCollector
    └── uses: CurrentDateProvider
```
