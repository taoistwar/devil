# Implementation Plan: Add Web Server Command

**Branch**: `260420-feat-add-web-server` | **Date**: 2026-04-20 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/009-add-web-server/spec.md`

## Summary

Add a `web` subcommand to the Devil CLI that starts an HTTP server exposing the agent's chat functionality via REST API. The server provides `/api/chat` for agent interactions and `/health` for health checks, with configurable host/port and graceful shutdown support.

## Technical Context

**Language/Version**: Rust 1.70+ (Edition 2021)
**Primary Dependencies**: tokio (async runtime), axum or std::net (HTTP server), serde (JSON)
**Storage**: N/A (stateless API)
**Testing**: cargo test, integration tests with reqwest
**Target Platform**: Linux server environments
**Project Type**: CLI tool with embedded web service
**Performance Goals**: Handle 10+ concurrent connections
**Constraints**: Graceful shutdown within 3 seconds
**Scale/Scope**: Single server instance, multiple concurrent API requests

## Constitution Check

| Gate | Status | Notes |
|------|--------|-------|
| Rust-First Standards | PASS | Using idiomatic Rust with Tokio |
| Tokio Concurrency | PASS | All async uses Tokio runtime |
| Claude Code Parity | N/A | New feature not in reference |
| Error Handling | PASS | Using anyhow::Result with context |
| Tool-First Architecture | PASS | CLI command exposing web API |

## Technical Approach

**HTTP Framework**: Use `axum` crate for HTTP server (lightweight, Tokio-native)

**Endpoints**:
- `POST /api/chat` - Accepts JSON `{"prompt": "..."}` and returns agent response
- `GET /health` - Returns `{"status": "ok"}`
- `GET /shutdown` - Triggers graceful shutdown

**Integration**: Reuse existing `Agent::run_once()` for processing prompts

**Command Registration**: Add `web` command to `Dispatcher` alongside `run`

## Project Structure

### This Feature

```text
specs/009-add-web-server/
├── plan.md              # This file
├── research.md          # N/A - straightforward web server
├── data-model.md        # API request/response structures
├── quickstart.md        # Usage examples
└── contracts/          # HTTP API contracts
    └── api.yaml        # OpenAPI-like contract
```

### Source Code Changes

```text
src/
├── cli/
│   ├── mod.rs           # Add web() function
│   └── dispatcher.rs    # Register "web" command
└── web/                # New module
    ├── mod.rs           # Web server entry point
    ├── handler.rs       # HTTP handlers
    └── server.rs        # Server setup and lifecycle
```

**Structure Decision**: Simple single-project structure. Adding a `web` module to the existing CLI binary. No new crates needed for v1.

## Complexity Tracking

No constitutional violations requiring justification.

## Implementation Notes

1. Use `axum` for HTTP server (tokio-native, lightweight)
2. JSON parsing with `serde`
3. Graceful shutdown via `tokio::signal::ctrl_c()`
4. Reuse existing `Agent::run_once()` for chat processing
5. API key authentication via `X-API-Key` header
