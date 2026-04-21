# Feature Specification: Add Web Server Command

**Feature Branch**: `260420-feat-add-web-server`
**Created**: 2026-04-20
**Status**: Draft
**Input**: User description: "添加 web 支持，和 run 同级，打开一个web服务"
**API Version**: v1

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start Web Server (Priority: P1)

Users can start a web server that exposes the Devil Agent functionality via HTTP API, allowing remote clients to interact with the agent.

**Why this priority**: This is the core functionality that enables the web service use case.

**Independent Test**: Can be fully tested by starting the server and sending an HTTP request to verify it responds.

**Acceptance Scenarios**:

1. **Given** the Devil CLI is installed, **When** the user runs `devil web`, **Then** a web server starts and listens on a configurable port
2. **Given** the web server is running, **When** a POST request is sent to `/api/v1/chat` with a prompt and valid API key, **Then** the agent processes the request and returns a response
3. **Given** the web server is running, **When** a GET request is sent to `/health`, **Then** a health check response is returned without authentication

---

### User Story 2 - Configure Web Server (Priority: P2)

Users can configure the web server port and other settings via command-line flags or environment variables.

**Why this priority**: Flexibility in deployment is important for different environments.

**Independent Test**: Can be tested by starting the server with different port configurations and verifying it listens on the correct port.

**Acceptance Scenarios**:

1. **Given** the CLI is installed, **When** the user runs `devil web --port 8080`, **Then** the server starts on port 8080
2. **Given** the CLI is installed, **When** the user runs `devil web --host 0.0.0.0`, **Then** the server binds to all network interfaces
3. **Given** the CLI is installed, **When** the user runs `devil web --api-key secret123`, **Then** the server requires this API key for authenticated endpoints

---

### User Story 3 - Stop Web Server (Priority: P2)

Users can gracefully stop the web server.

**Why this priority**: Proper shutdown is important for production deployments.

**Independent Test**: Can be tested by starting the server and then sending a shutdown signal, verifying the process terminates cleanly.

**Acceptance Scenarios**:

1. **Given** the web server is running, **When** the user presses Ctrl+C, **Then** the server shuts down gracefully
2. **Given** the web server is running, **When** a GET request is sent to `/api/v1/shutdown` with valid API key, **Then** the server stops

---

### User Story 4 - Rate Limiting (Priority: P2)

The API implements rate limiting to prevent abuse and ensure fair usage.

**Why this priority**: Essential for production deployment to prevent spam and ensure service stability.

**Independent Test**: Can be tested by sending requests beyond the rate limit and verifying 429 responses.

**Acceptance Scenarios**:

1. **Given** the API is configured with 60 requests per minute per API key, **When** a client exceeds this limit, **Then** the server returns 429 Too Many Requests with Retry-After header
2. **Given** multiple API keys are in use, **When** one key exceeds its limit, **Then** other keys are not affected

---

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `web` command alongside the existing `run` command
- **FR-002**: System MUST start an HTTP server when the `web` command is executed
- **FR-003**: System MUST expose a `/health` endpoint (no auth required) that returns server status
- **FR-004**: System MUST expose a `/api/v1/health` endpoint for detailed service health check
- **FR-005**: System MUST expose a `/api/v1/chat` endpoint that accepts POST requests with a prompt
- **FR-006**: System MUST accept `--port` flag to configure the listening port (default: 8080)
- **FR-007**: System MUST accept `--host` flag to configure the bind address (default: 127.0.0.1)
- **FR-008**: System MUST accept `--api-key` flag to configure the API authentication key (required for production)
- **FR-009**: System MUST gracefully shutdown when receiving SIGINT or SIGTERM
- **FR-010**: System MUST return appropriate HTTP error codes for malformed requests
- **FR-011**: System MUST support WebSocket connections at `/api/v1/ws` for streaming responses

### Authentication Requirements

- **AUTH-001**: All `/api/v1/*` endpoints (except `/api/v1/health` and `/api/v1/shutdown`) MUST require `X-API-Key` header
- **AUTH-002**: Invalid or missing API key MUST return 401 Unauthorized
- **AUTH-003**: API key MUST be configurable via `--api-key` flag or `DEVIL_API_KEY` environment variable

### Rate Limiting Requirements

- **RATE-001**: System MUST implement token bucket rate limiting per API key
- **RATE-002**: Default rate limit MUST be 60 requests per minute per API key
- **RATE-003**: Rate limited requests MUST return 429 Too Many Requests with `Retry-After` header
- **RATE-004**: Rate limit status MUST be queryable via `/api/v1/health` endpoint

### API Versioning Requirements

- **API-001**: All API endpoints MUST be versioned under `/api/v1/*`
- **API-002**: Version MUST be indicated in the `X-API-Version` response header
- **API-003**: Breaking changes MUST result in a new version (e.g., v2)

### Key Entities

- **WebServer**: Represents the HTTP server instance with host, port, and running state
- **ChatRequest**: Represents an incoming chat request with the user's prompt
- **ChatResponse**: Represents the agent's response to a chat request
- **RateLimiter**: Token bucket rate limiter per API key

## API v1 Endpoints

### Health Check Endpoints

#### GET /health

Simple liveness check for load balancers and orchestration systems.

**Authentication**: None required

**Response 200**:
```json
{
  "status": "ok"
}
```

#### GET /api/v1/health

Detailed service health check including rate limit status.

**Authentication**: Required (X-API-Key)

**Response 200**:
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "rate_limit": {
    "limit_per_minute": 60,
    "remaining": 45,
    "reset_at": "2026-04-20T12:00:00Z"
  }
}
```

### Chat Endpoint

#### POST /api/v1/chat

Send a chat prompt to the agent.

**Authentication**: Required (X-API-Key)

**Request Headers**:
```
Content-Type: application/json
X-API-Key: <your-api-key>
```

**Request Body**:
```json
{
  "prompt": "Hello, help me with task X",
  "session_id": "optional-session-id",
  "options": {
    "model": "optional-model-override",
    "temperature": 0.7
  }
}
```

**Response 200**:
```json
{
  "response": "I'll help you with task X...",
  "session_id": "uuid-of-session",
  "usage": {
    "input_tokens": 150,
    "output_tokens": 300
  }
}
```

**Response 401**:
```json
{
  "error": {
    "code": 1001,
    "message": "Invalid or missing API key"
  }
}
```

**Response 429**:
```json
{
  "error": {
    "code": 3003,
    "message": "Rate limit exceeded"
  },
  "retry_after_seconds": 30
}
```

### WebSocket Endpoint

#### GET /api/v1/ws

WebSocket connection for streaming chat responses.

**Authentication**: Required (X-API-Key as query parameter)

**URL**: `ws://host:port/api/v1/ws?api_key=<your-api-key>`

**Client Message**:
```json
{
  "type": "chat",
  "prompt": "Hello, help me with task X",
  "session_id": "optional-session-id"
}
```

**Server Messages**:
```json
{
  "type": "start",
  "session_id": "uuid"
}
{
  "type": "chunk",
  "content": "I'll help"
}
{
  "type": "chunk",
  "content": " you with"
}
{
  "type": "done",
  "usage": {
    "input_tokens": 150,
    "output_tokens": 300
  }
}
```

### Admin Endpoints

#### POST /api/v1/shutdown

Gracefully shutdown the server.

**Authentication**: Required (X-API-Key)

**Response 200**:
```json
{
  "message": "Shutting down..."
}
```

## Error Response Format

All API errors follow this format (per SPEC_DEPENDENCIES.md):

```json
{
  "error": {
    "code": 1001,
    "message": "Human readable error message",
    "field": "optional-field-name"
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| 1001 | 401 | Invalid or missing API key |
| 1002 | 400 | Missing required field |
| 1003 | 400 | Invalid request format |
| 2001 | 403 | Permission denied |
| 3003 | 429 | Rate limit exceeded |
| 5001 | 500 | Internal server error |

## Deployment Architecture

### Single Instance Deployment

```
                    ┌─────────────────┐
                    │   Load Balancer │
                    │  (optional)     │
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  Devil Web      │
                    │  Server         │
                    │  Port: 8080     │
                    └─────────────────┘
```

### Production Deployment Recommendations

1. **Reverse Proxy**: Deploy behind nginx/HAProxy for TLS termination
2. **Authentication**: Always use HTTPS in production
3. **Rate Limiting**: Configure appropriate limits based on expected traffic
4. **Monitoring**: Integrate with Prometheus/Grafana via `/metrics` endpoint (future)
5. **Process Manager**: Use systemd or similar for automatic restart

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DEVIL_API_KEY` | API authentication key | (none, required in production) |
| `DEVIL_WEB_PORT` | Server port | 8080 |
| `DEVIL_WEB_HOST` | Bind address | 127.0.0.1 |
| `DEVIL_RATE_LIMIT` | Requests per minute | 60 |

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can start the web server by running `devil web` and the server responds within 5 seconds
- **SC-002**: The `/api/v1/chat` endpoint returns responses for valid prompts with valid API key
- **SC-003**: The server handles at least 10 concurrent requests without failure
- **SC-004**: The server gracefully shuts down within 3 seconds of receiving SIGINT
- **SC-005**: Requests without valid API key receive 401 response
- **SC-006**: Requests exceeding rate limit receive 429 response with Retry-After header
- **SC-007**: WebSocket connections can established and receive streaming responses

## Edge Cases

- **Port Conflict**: Return error message indicating port is in use, suggest alternative port
- **Malformed JSON**: Return 400 with error code 1003
- **Missing API Key**: Return 401 with error code 1001
- **Invalid API Key**: Return 401 with error code 1001
- **Request Timeout**: Agent processing timeout returns 504 with error code 4003
- **Concurrent Limit**: When max concurrent requests reached, return 503 with retry hint
- **WebSocket Disconnect**: Clean up session resources on unexpected disconnect
- **Rate Limit Race**: Use atomic operations to ensure accurate rate limiting

## Assumptions

- Users have network access to connect to the web server
- The web interface is primarily for API access; browser-based UI is out of scope for v1
- Authentication for the web API will use API keys (simpler than OAuth)
- The existing agent core can be reused without modification for processing requests
- Rate limiting is per API key, not per IP address
