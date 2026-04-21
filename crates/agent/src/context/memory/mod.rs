//! Memory System Module
//!
//! Implements Claude Code's persistent, file-based memory system with typed memories.
//!
//! # Memory Types
//! - `user`: Information about user's role, goals, preferences
//! - `feedback`: Guidance on what to avoid or continue doing
//! - `project`: Work context, deadlines, decisions
//! - `reference`: Pointers to external systems
//!
//! # Directory Structure
//!
//! `<memory_dir>/` contains:
//! - `MEMORY.md` - Index file
//! - `user_*.md` - User memories
//! - `feedback_*.md` - Feedback memories
//! - `project_*.md` - Project memories
//! - `reference_*.md` - Reference memories

pub mod dir;
pub mod index;
pub mod prompts;
pub mod truncation;
pub mod types;

pub use dir::MemoryDir;
pub use index::{IndexEntry, MemoryIndex};
pub use prompts::MemoryGuidance;
pub use truncation::{truncate_entrypoint, EntrypointTruncation, MAX_ENTRYPOINT_BYTES, MAX_ENTRYPOINT_LINES};
pub use types::{parse_frontmatter, MemoryEntry, MemoryFrontmatter, MemoryType};
