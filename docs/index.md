# Forge — The Agent Harness SDK

**Self-improving harness with 4-layer pipeline: Observe → Detect → Strategize → Self-Improve. Works with ANY AI agent.**

Forge wraps around your existing agents (Claude Code, Cursor, LangGraph, CrewAI, AutoGen, raw APIs) and adds:

- **Layer 1: Observe** — 12-dimension real-time watching (token, latency, cost, accuracy, security, reliability, context, orch, comm, memory, compliance, diversity)
- **Layer 2: Detect** — 16 real-time detectors (loop, stale context, secret leak, hallucination, deadlock, cost anomaly...)
- **Layer 3: Strategize** — 14 autonomous strategies with priority-based selection (nudge, compact, pause, escalate, circuit break...)
- **Layer 4: Self-Improve** — Meta-harness mines weakness patterns across sessions, proposes rule edits, validates improvements
- **Immutable audit trail** — hash-chain integrity, full-text search, session replay
- **Full featured Dashboard** — UI-driven with tabbed analysis, 50+ API endpoints, WebSocket live streaming

## 🚀 Quick Start — The UI Way

```bash
pip install forge-agent-sdk    # One install
forge serve                     # Start dashboard
# Open http://localhost:3000    # Type task → Click Run → Watch live
```

The dashboard is the primary interface for Forge. Everything is UI-driven:
- **Run agents** with any of 31 presets (Claude Code, LangGraph, CrewAI, AutoGen, etc.)
- **Live monitoring** with real-time health gauges, conversation stream, intervention log
- **Compliance reports** — EU AI Act, SOC 2, ISO 27001, GDPR, HIPAA, PCI DSS
- **MCP management** — configure servers, discover local, start gateway
- **Skills browser** — built-in skills (security-first, cost-optimizer, accuracy-max)
- **Plugin marketplace** — search, install, publish community plugins
- **Cloud deployment** — AWS, Azure, GCP with one click
- **Analytics** — tokens, costs, interventions, health trends
- **Admin** — API keys, quotas, SSO configuration

## 🦀 Rust SDK

```rust
use forge_sdk::prelude::*;

// Your existing agent
let my_agent = MyClaudeAgent::new();

// Wrap in Forge harness
let harness = Harness::builder()
    .preset(Preset::Solo)
    .agent(my_agent)
    .build()?;

// Run — Forge watches, detects, intervenes
let session = harness.run("Implement JWT auth system").await?;
println!("{}", session.audit_report());
```

## 🐍 Python SDK

```python
from forge_sdk import create_harness

harness = create_harness(preset="claude-code")
result = harness.run("Write a function to validate email addresses")
print(f"Success: {result.success}")
print(f"Detections: {result.detection_count}")
print(f"Interventions: {result.intervention_count}")
```

## Why Forge?

| Without Forge | With Forge |
|---|---|
| Agent loops silently | Loop detected → nudge applied → agent breaks out |
| Context hits limit, session fails | Pressure detected at 75% → compacted → never hits limit |
| API key leaks in output | Secret leak detected → circuit break → session stopped |
| Agent hallucinates files | Hallucination detected → nudge → agent corrects |
| Cost spikes 10x | Cost anomaly detected → model swapped → savings |

## CLI Commands

```bash
forge serve           # Start dashboard + API server
forge run <task>      # Run agent through harness (CLI)
forge watch <id>      # Live TUI session viewer
forge explain <id>    # Human-readable audit report
forge init            # Scaffold new Forge project
forge doctor          # Check system dependencies
forge bench           # Run benchmark suite
forge improve         # Run meta-harness improvement
forge validate        # Validate harness config
```

## Installation

```bash
# Python (Recommended)
pip install forge-agent-sdk

# Rust
cargo install forge-sdk

# From source
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

## License

MIT
