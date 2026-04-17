# Feature Specification: CLI Entry Point Alignment

**Feature Branch**: `260417-feat-cli-entrypoint`
**Created**: 2026-04-17
**Status**: Draft
**Input**: User description: "对齐claude-code的入口功能：references/claude-code/src/entrypoints/cli.tsx"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Version Flag Fast Path (Priority: P1)

A developer types `devil --version` or `devil -v` and immediately sees the version number. The response is instant because no modules are loaded.

**Why this priority**: This is a standard CLI convention and provides immediate feedback. The fast path is critical for developer experience.

**Independent Test**: Can be tested by running `devil --version` and verifying output appears in under 100ms with no additional module loading.

**Acceptance Scenarios**:

1. **Given** the user runs `devil --version`, **When** the command executes, **Then** version string is printed and process exits cleanly with code 0
2. **Given** the user runs `devil -v`, **When** the command executes, **Then** version string is printed and process exits cleanly with code 0
3. **Given** the user runs `devil -V`, **When** the command executes, **Then** version string is printed and process exits cleanly with code 0

---

### User Story 2 - Help System (Priority: P1)

A developer types `devil --help` or `devil help` and sees comprehensive help information including all available commands, flags, and examples.

**Why this priority**: Help is the primary discovery mechanism for new users and a reference for existing users.

**Independent Test**: Can be tested by running `devil --help` and verifying all commands and options are listed correctly.

**Acceptance Scenarios**:

1. **Given** the user runs `devil --help`, **When** the command executes, **Then** help text displays all available commands
2. **Given** the user runs `devil help`, **When** the command executes, **Then** help text displays identically to `--help`
3. **Given** the user runs `devil` with no arguments, **When** the command executes, **Then** help text is displayed

---

### User Story 3 - Single Task Execution Mode (Priority: P1)

A developer provides a prompt as a command-line argument and the agent executes the task without entering interactive mode.

**Why this priority**: This enables scripting and automation where a full REPL session is not needed.

**Independent Test**: Can be tested by running `devil run "analyze project structure"` and verifying the agent completes the task.

**Acceptance Scenarios**:

1. **Given** the user runs `devil run "task description"`, **When** the command executes, **Then** the agent processes the task and exits upon completion
2. **Given** the user provides multiple words in quotes, **When** the command executes, **Then** all words are treated as a single task description
3. **Given** the user runs `devil run` without a task, **When** the command executes, **Then** an error message explains the correct usage

---

### User Story 4 - Interactive REPL Mode (Priority: P1)

A developer types `devil repl` and enters an interactive read-eval-print loop where they can converse with the agent in real-time.

**Why this priority**: The REPL is the primary interactive interface for complex, multi-turn conversations.

**Independent Test**: Can be tested by running `devil repl` and sending test messages to verify responses.

**Acceptance Scenarios**:

1. **Given** the user runs `devil repl`, **When** the command executes, **Then** the terminal switches to interactive mode with a prompt
2. **Given** the user is in REPL mode, **When** they type a message and press Enter, **Then** the agent responds
3. **Given** the user presses Ctrl+C in REPL mode, **When** the interrupt is received, **Then** the session can continue or exit gracefully

---

### User Story 5 - Configuration Management (Priority: P2)

A developer can view and modify configuration settings through the `devil config` command.

**Why this priority**: Configuration management enables users to customize agent behavior and manage API credentials.

**Independent Test**: Can be tested by running `devil config show` and verifying configuration values are displayed.

**Acceptance Scenarios**:

1. **Given** the user runs `devil config show`, **When** the command executes, **Then** current configuration is displayed
2. **Given** the user runs `devil config set KEY VALUE`, **When** the command executes, **Then** the configuration value is updated
3. **Given** the user runs `devil config get KEY`, **When** the command executes, **Then** the specific value is displayed

---

### User Story 6 - Dynamic Command Dispatch (Priority: P1)

The CLI dispatcher routes commands to appropriate handlers based on command-line arguments, supporting both subcommands and flags.

**Why this priority**: A well-structured dispatcher is the foundation for adding new commands without modifying core logic.

**Independent Test**: Can be tested by verifying each command routes to its handler without interference.

**Acceptance Scenarios**:

1. **Given** the CLI receives `devil version`, **When** the dispatcher parses the command, **Then** the version handler is invoked
2. **Given** the CLI receives `devil --version` as a flag, **When** the dispatcher parses the arguments, **Then** the version handler is invoked (flag takes precedence)
3. **Given** the CLI receives an unknown command, **When** the dispatcher cannot find a handler, **Then** an error message is shown and process exits with code 1

---

### User Story 7 - Environment Variable Configuration (Priority: P2)

The CLI loads configuration from environment variables, applying them before and after config file settings.

**Why this priority**: Environment variables enable containerized deployments and CI/CD integration.

**Independent Test**: Can be tested by setting environment variables and verifying they override config file values.

**Acceptance Scenarios**:

1. **Given** `DEVIL_API_KEY` is set in environment, **When** the agent starts, **Then** the API key is loaded from the environment variable
2. **Given** both config file and environment variable specify the same key, **When** initialization occurs, **Then** environment variable takes precedence

---

### User Story 8 - Graceful Shutdown (Priority: P2)

When the process receives termination signals (SIGINT, SIGTERM), it cleans up resources properly before exiting.

**Why this priority**: Graceful shutdown prevents data loss and resource leaks during planned outages.

**Independent Test**: Can be tested by sending SIGTERM to a running process and verifying clean exit.

**Acceptance Scenarios**:

1. **Given** the agent is running, **When** it receives SIGINT (Ctrl+C), **Then** cleanup handlers execute and process exits cleanly
2. **Given** the agent is running, **When** it receives SIGTERM, **Then** cleanup handlers execute and process exits cleanly with code 0

---

### Edge Cases

- What happens when the user provides an empty string as the task?
- How does the system handle extremely long command-line arguments?
- What happens when stdin is not a TTY (piped input)?
- How does the system behave when HOME environment variable is not set?
- What happens when the config file is malformed or unreadable?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: The CLI MUST respond to `--version`, `-v`, and `-V` flags with version string in under 100ms
- **FR-002**: The CLI MUST display help when run with no arguments, `--help`, or `help`
- **FR-003**: The CLI MUST support `run <prompt>` for single-task execution mode
- **FR-004**: The CLI MUST support `repl` for interactive mode
- **FR-005**: The CLI MUST support `config` subcommand for configuration management
- **FR-006**: The CLI MUST route commands via a central dispatcher pattern
- **FR-007**: The CLI MUST load environment variables for configuration before config file
- **FR-008**: The CLI MUST register cleanup handlers for graceful shutdown
- **FR-009**: The CLI MUST handle SIGINT and SIGTERM for graceful termination
- **FR-010**: Error messages MUST be written to stderr and exit codes MUST be non-zero for errors
- **FR-011**: The CLI version MUST be compile-time constant injected via build

### Key Entities

- **Command**: A CLI command with name, description, handler function, and argument schema
- **Config**: Key-value store of agent configuration loaded from files and environment
- **InitState**: Runtime state initialized at startup including telemetry, logging, and cleanup handlers
- **CliContext**: Shared context passed to command handlers containing config and runtime state

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: `devil --version` returns in under 100ms with zero module loading
- **SC-002**: `devil --help` displays all commands with descriptions
- **SC-003**: Unknown commands return exit code 1 with error message
- **SC-004**: Graceful shutdown completes within 2 seconds of signal receipt
- **SC-005**: All commands have corresponding integration tests
- **SC-006**: Version string matches Cargo.toml package version

## Assumptions

- Users have Rust 1.70+ and Cargo installed
- Tokio runtime is available for async operations
- Config files use JSON or TOML format
- Environment variable prefix is `DEVIL_`
- Default config location is `~/.devil/config.toml`
- Session state is stored in `~/.devil/sessions/`
