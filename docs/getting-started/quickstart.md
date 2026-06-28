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
2. Select agent type: `Solo` (mock agent, no API key needed)
3. Click **Run with Harness**

The dashboard opens the live session view. You'll see:
- The agent's conversation stream in real-time
- Health gauges updating (token, latency, cost, accuracy, security...)
- Interventions appear when the harness detects issues

## 4. Explore

Once the session completes:
- **Sessions** — see your session in the history table with tabbed analysis
- **Audit** — browse the immutable audit trail
- **Settings** — toggle observers, detectors, and strategies

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
- **Rust SDK**: Build from source for custom integrations — see [Installation](installation.md)
- **Architecture**: Understand the 4-layer pipeline — see [Architecture](../concepts/architecture.md)
