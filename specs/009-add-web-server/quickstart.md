# Quickstart: Web Server

## Starting the Server

```bash
# Default (127.0.0.1:8080)
devil web

# Custom port
devil web --port 3000

# Bind to all interfaces
devil web --host 0.0.0.0 --port 8080
```

## API Usage

### Chat

```bash
curl -X POST http://localhost:8080/api/chat \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{"prompt": "Hello, how are you?"}'
```

**Response**:
```json
{
  "response": "I'm doing great! How can I help you today?",
  "success": true,
  "turns": 1,
  "terminal_reason": "Completed"
}
```

### Health Check

```bash
curl http://localhost:8080/health
```

**Response**:
```json
{
  "status": "ok",
  "version": "0.1.0"
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| DEVIL_API_KEY | API key for authentication | (none) |
| DEVIL_PORT | Server port | 8080 |
| DEVIL_HOST | Bind address | 127.0.0.1 |

## Shutdown

```bash
# Send Ctrl+C or:
curl http://localhost:8080/shutdown
```
