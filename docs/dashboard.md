# Forge Dashboard

The Forge Dashboard is a web-based UI that gives you full control over every Forge feature. No terminal commands needed.

## Starting the Dashboard

```bash
forge serve
# Opens http://localhost:3000
```

## Pages

### Run
Type your task, select agent type (31 options) and preset, click Run. Watch the live session stream with real-time health gauges.

### Sessions
View all past and active sessions with Tokens and Model columns. Filter by status, agent type, or date. Click any session for tabbed full-context analysis:

- **Overview** — session summary, health score, recommendations
- **Timeline** — every event with seq#, time, type, tool, args, result, duration
- **Tools & Prompts** — every tool call with extracted args/results; every prompt with following tool count, latency, tokens
- **Detections** — issues grouped by category with severity, confidence, descriptions
- **Hooks & Context** — events grouped by hook type; context pressure history over time

### Live
WebSocket-connected real-time view of a running session. Shows conversation stream, 12-dimension health gauges, and intervention log. Pause/Resume controls.

### Audit
Browse immutable audit events. Filter by phase (Observe/Detect/Strategy/Action), severity, detector. Full-text search. Export to JSON/CSV/PDF.

### Compliance
Generate compliance reports for EU AI Act (Article 14 + 15), SOC 2 Type II, ISO 27001, GDPR, HIPAA, PCI DSS. Shows checklist with pass/fail per requirement.

### Skills
Browse built-in skills (security-first, cost-optimizer, accuracy-max). Compose multiple skills to merge observer/detector/strategy configurations.

### MCP
Configure MCP servers (stdio, SSE, HTTP). Discover local servers. Start MCP gateway. Expose Forge as an MCP server with tools and resources.

### Export
Configure 10 export targets: LangFuse, Weights & Biases, OpenTelemetry, PagerDuty, OpsGenie, Splunk, Elasticsearch, Datadog, Slack, Discord. Set alerting rules.

### Marketplace
Search community plugins by type (Observer, Detector, Strategy, Skill, Preset, Adapter). Install with one click. Publish your own.

### Cloud
Configure AWS, Azure, GCP providers. Select regions. Deploy Forge infrastructure with one click.

### Analytics
Overview stats (sessions, tokens, cost, health). Token breakdown by model. Cost by agent. Intervention success rates. Health trends over time.

### Meta
Harness version history. Pass rate improvement chart. Discovered weakness patterns. Pending edits with accept/reject. A/B test results.

### Auth
Configure SSO (Okta, Azure AD, Google Workspace, Custom OIDC). Group-to-role mapping. JIT provisioning. MFA toggle. Session duration.

### Admin
Create and revoke API keys with scopes (read:audit, write:agent, admin). Set usage quotas (sessions/day, tokens/month, cost/month).

### Settings
Toggle individual observers (12), detectors (16), and strategies (14). Adjust detector thresholds. Switch presets. Import/Export harness config.

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

# Generate compliance report
curl "http://localhost:3000/api/v1/compliance/report?framework=EuAiAct&session_id=abc123"

# List skills
curl http://localhost:3000/api/v1/skills

# Get harness config
curl http://localhost:3000/api/v1/harness
```

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
