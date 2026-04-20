# Research: Web Server Implementation

## Decision: HTTP Framework Selection

**Chosen**: `axum` crate

**Rationale**:
- Tokio-native: Built from scratch for Tokio, no blocking
- Lightweight: Minimal abstraction over hyper
- Type-safe routing: Compile-time guarantee routes exist
- Middleware support: Easy to add auth, logging

**Alternatives considered**:
- `actix-web`: More mature but heavier, different programming model
- `tiny_http`: Too low-level, manual routing needed
- `poem`: Good but less Tokio-native

## Decision: JSON Library

**Chosen**: `serde` + `serde_json`

**Rationale**: Already in workspace dependencies, stable and fast.

## Decision: Authentication

**Chosen**: API Key via `X-API-Key` header

**Rationale**: Simple to implement, matches existing `DEVIL_API_KEY` pattern. OAuth2 is overkill for v1.

## Decision: Concurrency Model

**Chosen**: Tokio task per request

**Rationale**: Natural fit for tokio, simple to reason about. Agent processing is async so no blocking.
