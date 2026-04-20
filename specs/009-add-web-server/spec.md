# Feature Specification: Add Web Server Command

**Feature Branch**: `260420-feat-add-web-server`
**Created**: 2026-04-20
**Status**: Draft
**Input**: User description: "添加 web 支持，和 run 同级，打开一个web服务"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Start Web Server (Priority: P1)

Users can start a web server that exposes the Devil Agent functionality via HTTP API, allowing remote clients to interact with the agent.

**Why this priority**: This is the core functionality that enables the web service use case.

**Independent Test**: Can be fully tested by starting the server and sending an HTTP request to verify it responds.

**Acceptance Scenarios**:

1. **Given** the Devil CLI is installed, **When** the user runs `devil web`, **Then** a web server starts and listens on a configurable port
2. **Given** the web server is running, **When** a POST request is sent to `/api/chat` with a prompt, **Then** the agent processes the request and returns a response
3. **Given** the web server is running, **When** a GET request is sent to `/health`, **Then** a health check response is returned

---

### User Story 2 - Configure Web Server (Priority: P2)

Users can configure the web server port and other settings via command-line flags or environment variables.

**Why this priority**: Flexibility in deployment is important for different environments.

**Independent Test**: Can be tested by starting the server with different port configurations and verifying it listens on the correct port.

**Acceptance Scenarios**:

1. **Given** the CLI is installed, **When** the user runs `devil web --port 8080`, **Then** the server starts on port 8080
2. **Given** the CLI is installed, **When** the user runs `devil web --host 0.0.0.0`, **Then** the server binds to all network interfaces

---

### User Story 3 - Stop Web Server (Priority: P2)

Users can gracefully stop the web server.

**Why this priority**: Proper shutdown is important for production deployments.

**Independent Test**: Can be tested by starting the server and then sending a shutdown signal, verifying the process terminates cleanly.

**Acceptance Scenarios**:

1. **Given** the web server is running, **When** the user presses Ctrl+C, **Then** the server shuts down gracefully
2. **Given** the web server is running, **When** a GET request is sent to `/shutdown`, **Then** the server stops

---

### Edge Cases

- What happens when the port is already in use?
- How does the system handle malformed requests?
- What happens when the agent processing takes too long?
- How does the server handle concurrent requests?

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a `web` command alongside the existing `run` command
- **FR-002**: System MUST start an HTTP server when the `web` command is executed
- **FR-003**: System MUST expose a `/api/chat` endpoint that accepts POST requests with a prompt
- **FR-004**: System MUST expose a `/health` endpoint that returns server status
- **FR-005**: System MUST accept `--port` flag to configure the listening port (default: 8080)
- **FR-006**: System MUST accept `--host` flag to configure the bind address (default: 127.0.0.1)
- **FR-007**: System MUST gracefully shutdown when receiving SIGINT or SIGTERM
- **FR-008**: System MUST return appropriate HTTP error codes for malformed requests

### Key Entities

- **WebServer**: Represents the HTTP server instance with host, port, and running state
- **ChatRequest**: Represents an incoming chat request with the user's prompt
- **ChatResponse**: Represents the agent's response to a chat request

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can start the web server by running `devil web` and the server responds within 5 seconds
- **SC-002**: The `/api/chat` endpoint returns responses for valid prompts
- **SC-003**: The server handles at least 10 concurrent requests without failure
- **SC-004**: The server gracefully shuts down within 3 seconds of receiving SIGINT

## Assumptions

- Users have network access to connect to the web server
- The web interface is primarily for API access; browser-based UI is out of scope for v1
- Authentication for the web API will use API keys (simpler than OAuth)
- The existing agent core can be reused without modification for processing requests
