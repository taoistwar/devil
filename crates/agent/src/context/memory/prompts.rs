//! Memory Guidance Prompts
//!
//! Static guidance text from Claude Code's memory.ts.
//! Includes guidance on what to save, what NOT to save, and when to access.

/// Memory guidance builder
pub struct MemoryGuidance;

impl MemoryGuidance {
    /// Build complete memory prompt (without MEMORY.md content)
    pub fn build_prompt(_memory_dir: &str) -> Vec<&'static str> {
        let mut lines = Self::TYPES_SECTION_INDIVIDUAL.to_vec();
        lines.push("");
        lines.extend_from_slice(Self::WHAT_NOT_TO_SAVE);
        lines.push("");
        lines.extend_from_slice(Self::WHEN_TO_ACCESS);
        lines.push("");
        lines.extend_from_slice(Self::TRUSTING_RECALL);
        lines
    }

    /// Types section (individual mode)
    pub const TYPES_SECTION_INDIVIDUAL: &'static [&'static str] = &[
        "## Types of memory",
        "",
        "There are several discrete types of memory that you can store in your memory system:",
        "",
        "<types>",
        "<type>",
        "    <name>user</name>",
        "    <description>Contain information about the user's role, goals, responsibilities, and knowledge. Great user memories help you tailor your future behavior to the user's preferences and perspective.</description>",
        "    <when_to_save>When you learn any details about the user's role, preferences, responsibilities, or knowledge</when_to_save>",
        "    <how_to_use>When your work should be informed by the user's profile or perspective.</how_to_use>",
        "</type>",
        "<type>",
        "    <name>feedback</name>",
        "    <description>Guidance the user has given you about how to approach work — both what to avoid and what to keep doing. Record from failure AND success.</description>",
        "    <when_to_save>Any time the user corrects your approach OR confirms a non-obvious approach worked.</when_to_save>",
        "    <how_to_use>Let these memories guide your behavior so that the user does not need to offer the same guidance twice.</how_to_use>",
        "</type>",
        "<type>",
        "    <name>project</name>",
        "    <description>Information that you learn about ongoing work, goals, initiatives, bugs, or incidents within the project that is not otherwise derivable from the code or git history.</description>",
        "    <when_to_save>When you learn who is doing what, why, or by when. Always convert relative dates to absolute dates when saving.</when_to_save>",
        "    <how_to_use>Use these memories to more fully understand the details and nuance behind the user's request.</how_to_use>",
        "</type>",
        "<type>",
        "    <name>reference</name>",
        "    <description>Stores pointers to where information can be found in external systems.</description>",
        "    <when_to_save>When you learn about resources in external systems and their purpose.</when_to_save>",
        "    <how_to_use>When the user references an external system or information that may be in an external system.</how_to_use>",
        "</type>",
        "</types>",
        "",
    ];

    /// What NOT to save section
    pub const WHAT_NOT_TO_SAVE: &'static [&'static str] = &[
        "## What NOT to save in memory",
        "",
        "- Code patterns, conventions, architecture, file paths, or project structure — these can be derived by reading the current project state.",
        "- Git history, recent changes, or who-changed-what — `git log` / `git blame` are authoritative.",
        "- Debugging solutions or fix recipes — the fix is in the code; the commit message has the context.",
        "- Anything already documented in CLAUDE.md files.",
        "- Ephemeral task details: in-progress work, temporary state, current conversation context.",
        "",
        "These exclusions apply even when the user explicitly asks you to save. If they ask you to save a PR list or activity summary, ask what was *surprising* or *non-obvious* about it — that is the part worth keeping.",
    ];

    /// When to access section
    pub const WHEN_TO_ACCESS: &'static [&'static str] = &[
        "## When to access memories",
        "- When memories seem relevant, or the user references prior-conversation work.",
        "- You MUST access memory when the user explicitly asks you to check, recall, or remember.",
        "- If the user says to *ignore* or *not use* memory: proceed as if MEMORY.md were empty. Do not apply remembered facts, cite, compare against, or mention memory content.",
        "- Memory records can become stale over time. Before answering the user or building assumptions based solely on information in memory records, verify that the memory is still correct.",
    ];

    /// Trusting recall section
    pub const TRUSTING_RECALL: &'static [&'static str] = &[
        "## Before recommending from memory",
        "",
        "A memory that names a specific function, file, or flag is a claim that it existed *when the memory was written*. It may have been renamed, removed, or never merged. Before recommending it:",
        "",
        "- If the memory names a file path: check the file exists.",
        "- If the memory names a function or flag: grep for it.",
        "- If the user is about to act on your recommendation (not just asking about history), verify first.",
        "",
        "\"The memory says X exists\" is not the same as \"X exists now.\"",
        "",
        "A memory that summarizes repo state (activity logs, architecture snapshots) is frozen in time. If the user asks about *recent* or *current* state, prefer `git log` or reading the code over recalling the snapshot.",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt() {
        let prompt = MemoryGuidance::build_prompt("/test/path");
        assert!(!prompt.is_empty());
        assert!(prompt.contains(&"## Types of memory"));
        assert!(prompt.contains(&"## What NOT to save in memory"));
    }

    #[test]
    fn test_types_section_present() {
        let prompt = MemoryGuidance::build_prompt("");
        let prompt_str = prompt.join("\n");
        assert!(prompt_str.contains("<name>user</name>"));
        assert!(prompt_str.contains("<name>feedback</name>"));
        assert!(prompt_str.contains("<name>project</name>"));
        assert!(prompt_str.contains("<name>reference</name>"));
    }
}
