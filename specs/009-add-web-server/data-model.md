# Data Model: Web Server API

## Entities

### ChatRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| prompt | string | Yes | The user's prompt to the agent |
| stream | boolean | No | Enable streaming response (future) |

### ChatResponse

| Field | Type | Description |
|-------|------|-------------|
| response | string | The agent's text response |
| success | boolean | Whether processing succeeded |
| error | string | Error message if success=false |
| turns | number | Number of agent turns taken |
| terminal_reason | string | Why the conversation ended |

### HealthResponse

| Field | Type | Description |
|-------|------|-------------|
| status | string | "ok" when healthy |
| version | string | CLI version |

## State Transitions

**WebServer States**:
- `Starting` → Server initializing
- `Running` → Accepting requests
- `ShuttingDown` → Graceful shutdown in progress
- `Stopped` → Server closed

**Terminal States**: `Completed`, `MaxTurns`, `Error`, `UserAborted`

## Validation Rules

- `prompt`: Max 100,000 characters
- `stream`: Must be boolean if present
