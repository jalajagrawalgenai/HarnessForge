# Quickstart

Get Forge running in under 2 minutes.

## 1. Install

```bash
pip install forge-agent-sdk
```

Requirements: Python 3.10+, Windows/macOS/Linux.

## 2. Launch the Dashboard

```bash
forge serve
```

Opens `http://localhost:3000` — the Forge Dashboard.

## 3. Run Your First Agent

1. In the **Run** page, type a task: `"Write a function to validate email addresses"`
2. Select agent type: `Claude Code` (or `Solo` for testing without an API key)
3. Select preset: `solo`
4. Click **Run with Harness**

The dashboard opens the live session view. You'll see:
- The agent's conversation stream in real-time
- 12 health gauges updating (token, latency, cost, accuracy, security...)
- Interventions appear when the harness detects issues

## 4. Explore

Once the session completes:
- **Sessions** — see your session in the history table
- **Audit** — browse the immutable audit trail
- **Compliance** — generate an EU AI Act report for your session
- **Analytics** — see aggregate stats across all sessions

## 5. What Forge Catches

Check the intervention log to see what the harness detected:
```
⚠ HARNESS [T4]: StaleContext detected. file.rs re-read 4 times.
   → CompactStrategy: Context 84% → 58%. Saved 3.2K tokens.

🔧 HARNESS [T7]: AccuracyRisk detected. Code generated, no tests.
   → NudgeStrategy: "Run the tests before proceeding."
```

## Next Steps

- **Python SDK**: `from forge_sdk import create_harness` — programmatic access
- **Rust SDK**: `use forge_sdk::Harness` — build custom integrations
- **Guides**: Add custom observers, detectors, or strategies
- **MCP**: Configure Model Context Protocol servers
- **Skills**: Install community skills or create your own
