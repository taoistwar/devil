<!--
  Sync Impact Report:
  Version change: 0.1.0 → 1.0.0
  Modified principles:
    - [NEW] I. Rust-First Standards (added)
    - [NEW] II. Tokio Concurrency Model (added)
    - [NEW] III. Claude Code Reference Parity (added)
    - [NEW] IV. Robust Error Handling (added)
    - [NEW] V. Tool-First Architecture (added)
  Added sections:
    - Technology Stack Constraints
    - Code Quality Standards
  Removed sections: None
  Templates requiring updates: ⚠️ pending review
    - .specify/templates/plan-template.md (Constitution Check section aligns)
    - .specify/templates/spec-template.md (no changes needed)
    - .specify/templates/tasks-template.md (no changes needed)
  Deferred items: None
-->

# MonkeyCode AI Agent Constitution

## Core Principles

### I. Rust-First Standards (NON-NEGOTIABLE)

Rust is the sole implementation language. All code MUST be idiomatic Rust following best practices.

- All code MUST compile without warnings
- All code MUST pass `cargo clippy` with zero warnings (allow list required for exceptions)
- Use ownership and borrowing correctly; no unsafe code without explicit justification and review
- Generic code preferred over duplication; trait objects only when type erasure is necessary
- Error types MUST be descriptive and include context via `anyhow::Context` or custom `thiserror` types

### II. Tokio Concurrency Model (NON-NEGOTIABLE)

All asynchronous operations MUST use the Tokio runtime.

- All async code MUST use Tokio's task spawning, channels, and timing primitives
- `#[tokio::main]` or `#[tokio::test]` MUST be used for main/test entry points
- Blocking operations MUST be wrapped in `tokio::task::spawn_blocking`
- All traits that define async methods MUST use `#[async_trait]`
- Deadlocks and race conditions are considered critical bugs

### III. Claude Code Reference Parity (NON-NEGOTIABLE)

The Rust implementation MUST maintain logical parity with the `references/claude-code` implementation.

- Core tool semantics (Bash, Read, Edit, Write, Glob, Grep) MUST match reference behavior
- Tool attribute semantics (`isReadOnly`, `isDestructive`, `isConcurrencySafe`) MUST match reference
- Permission checking behavior MUST follow reference patterns
- When implementation differs from reference, explicit justification required in code comments
- The five-factor protocol (Input, Output, Progress, Permissions, Metadata) MUST be implemented

### IV. Robust Error Handling (NON-NEGOTIABLE)

All error handling MUST be explicit, recoverable, and informative.

- Use `anyhow::Result<T>` for application-level error handling with context
- Use `thiserror` for library/API error types that require specific variant handling
- Never panic; all panics MUST be replaced with proper error propagation
- Errors MUST include context: file paths, line numbers, operation descriptions
- Destructors MUST NOT panic; use `std::panic::catch_unwind` for FFI boundaries

### V. Tool-First Architecture

Every library MUST expose functionality via CLI tools before other interfaces.

- All core libraries MUST have corresponding CLI tools
- Text in/out protocol: stdin/args → stdout, errors → stderr
- Support both JSON (machine-readable) and human-readable output formats
- Tools MUST be independently testable with clear contracts

## Technology Stack Constraints

- **Language**: Rust (Edition 2021+)
- **Async Runtime**: Tokio (mandatory for all async operations)
- **Error Handling**: `anyhow` for application errors, `thiserror` for library errors
- **Minimum Rust Version**: Rust 1.70+
- **Target Platform**: Linux server environments

## Code Quality Standards

### Testing Requirements

- Unit tests REQUIRED for all public APIs
- Integration tests REQUIRED for inter-crate communication
- `cargo test` MUST pass with 100% success rate
- Code coverage SHOULD be measured; significant drops require justification

### Linting and Formatting

- `cargo fmt` MUST be run before any commit
- `cargo clippy` MUST pass with zero warnings (allow exceptions documented)
- No dead code; unused imports, functions, and variables are errors
- Documentation comments (`///`) REQUIRED for all public items

### Performance Requirements

- Memory allocation patterns MUST avoid unnecessary clones
- Large data structures SHOULD implement `Send + Sync` when safe
- Async task cancellation MUST be handled gracefully
- Resource cleanup MUST be guaranteed (RAII patterns preferred)

## Security Requirements

### Permission Model

- All tools MUST declare read/write/destructive behavior
- Destructive operations require explicit user confirmation
- Permission rules MUST be checked before tool execution
- `.gitignore` and exclusion patterns MUST be respected

### Input Validation

- All external input MUST be validated before processing
- Path traversal attacks MUST be prevented
- Shell command injection MUST be blocked via AST parsing
- Device file access MUST be restricted

## Development Workflow

### Phase Order

1. **Research**: Understand reference implementation and dependencies
2. **Design**: Document data structures and API contracts
3. **Test-First**: Write tests before implementation
4. **Implement**: Write idiomatic Rust passing clippy
5. **Verify**: All tests pass, documentation complete

### Code Review Requirements

- All PRs MUST verify clippy cleanliness
- Constitution compliance MUST be checked
- New unsafe code requires explicit security review
- Breaking changes MUST be documented and justified

## Governance

This constitution supersedes all other development practices. 

### Amendment Procedure

1. Proposed changes MUST be documented with rationale
2. Changes MUST be reviewed for constitutional compliance
3. MAJOR versions require team approval; MINOR/PATCH can proceed with review
4. Version bumps: MAJOR for principle removals, MINOR for additions, PATCH for clarifications

### Compliance Verification

- All PRs/reviews MUST verify constitution compliance
- Complexity MUST be justified against simpler alternatives
- Use `.specify/memory/constitution.md` as the authoritative source

**Version**: 1.0.0 | **Ratified**: 2026-04-17 | **Last Amended**: 2026-04-17
