# Forge Dashboard

The Forge Dashboard is a web-based UI for monitoring and managing your AI agent sessions.

## Starting the Dashboard

```bash
forge serve
# Opens http://localhost:3000
```

## Pages

### Run
Type your task, select agent type (31 presets available), and click Run. Watch the live session stream with real-time health gauges.

### Sessions
View all past and active sessions. Filter by status, agent type, or date. Click any session for tabbed full-context analysis:

- **Overview** — session summary, health score, recommendations
- **Timeline** — every event with seq#, time, type, tool, args, result, duration
- **Tools & Prompts** — every tool call with extracted args/results; every prompt with following tool count, latency, tokens
- **Detections** — issues grouped by category with severity, confidence, descriptions
- **Hooks & Context** — events grouped by hook type; context pressure history over time

### Live
WebSocket-connected real-time view of a running session. Shows conversation stream, health gauges, and intervention log.

### Audit
Browse immutable audit events. Filter by phase (Observe/Detect/Strategy/Action), severity, detector. Full-text search. Export to JSON/CSV.

### Settings
Toggle individual observers (12), detectors (16), and strategies (14). Adjust detector thresholds. Switch presets.

## API

The dashboard consumes the REST API at `/api/v1/*`. The same API is available for programmatic use:

```bash
# Health check
curl http://localhost:3000/api/v1/health

# Create session
curl -X POST http://localhost:3000/api/v1/sessions \
  -H "Content-Type: application/json" \
  -d '{"task":"Write fibonacci","agent_type":"solo","preset":"solo"}'

# List sessions
curl http://localhost:3000/api/v1/sessions

# Get harness config
curl http://localhost:3000/api/v1/harness
```

Additional endpoints are available under `/api/v1/` for compliance, skills, MCP, analytics, meta, and more.

## WebSocket

Live session events stream over WebSocket at `ws://localhost:3000/ws`:

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');
ws.onopen = () => ws.send(JSON.stringify({action: 'subscribe', session_id: '...'}));
ws.onmessage = (e) => console.log(JSON.parse(e.data)); // AgentEvent
```

## Architecture

```
Browser (Dashboard) ──→ REST API (/api/v1/*) ──→ forge-harness
                    ──→ WebSocket (/ws) ──→ broadcast events
                    ──→ Static Files (/) ──→ SPA
```
