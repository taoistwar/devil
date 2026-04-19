# Quickstart: Claude Code Context Alignment

## Overview

This feature adds context injection aligned with Claude Code's `context.ts`:
- Git status collection (branch, main branch, status, recent commits)
- Current date injection (ISO 8601 format)
- Memory files (CLAUDE.md) discovery and loading
- Context caching with cache breaker support

## Usage

### Automatic Context Injection

Context is automatically collected when the agent starts a conversation:

```rust
use crate::context::{system_context, user_context};

// System context (git status + cache breaker)
let system = system_context::get_system_context().await;

// User context (memory files + date)
let user = user_context::get_user_context().await;
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CLAUDE_CODE_DISABLE_CLAUDE_MDS` | Set to `1` to disable Memory files |
| `CLAUDE_CODE_REMOTE` | Set to `1` to skip git status (remote sessions) |

### Bare Mode

In `--bare` mode, automatic memory file discovery is skipped unless directories are explicitly added:

```rust
// With --bare and no explicit dirs: skip discovery
// With --bare and explicit dirs: still process those dirs
let ctx = user_context::get_user_context().await;
```

### Cache Breaker

To force cache refresh:

```rust
use crate::context::system_context::{set_system_prompt_injection, get_system_context};

set_system_prompt_injection("debug:force_refresh".to_string());
let ctx = get_system_context().await; // Cache cleared
```

## Testing

```bash
# Run context module tests
cargo test -p agent context

# Run specific test
cargo test -p agent git_status
```

## Configuration

No external configuration required. All behavior is controlled via environment variables and code.
