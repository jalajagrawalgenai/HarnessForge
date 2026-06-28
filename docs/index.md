# Forge — The Agent Harness SDK

**Self-improving harness with 4-layer pipeline: Observe → Detect → Strategize → Self-Improve. Works with ANY AI agent.**

Forge wraps around your existing agents (Claude Code, Cursor, LangGraph, CrewAI, AutoGen, raw APIs) and adds:

- **Layer 1: Observe** — 12-dimension real-time watching (token, latency, cost, accuracy, security, reliability, context, orch, comm, memory, compliance, diversity)
- **Layer 2: Detect** — 16 real-time detectors (loop, stale context, secret leak, hallucination, deadlock, cost anomaly...)
- **Layer 3: Strategize** — 14 autonomous strategies with priority-based selection (nudge, compact, pause, escalate, circuit break...)
- **Layer 4: Self-Improve** — Meta-harness mines weakness patterns across sessions, proposes rule edits, validates improvements
- **Immutable audit trail** — hash-chain integrity, full-text search, session replay
- **Dashboard** — UI-driven with tabbed analysis, REST + WebSocket API, live streaming
- **Hook auto-registration** — automatically registers with Claude Code on first import

## 🚀 Quick Start

```bash
pip install forge-agent-sdk    # One install
forge serve                     # Start dashboard at http://localhost:3000
```

The dashboard is the primary Forge interface:

- **Run agents** with any of 31 presets (Claude Code, LangGraph, CrewAI, AutoGen, etc.)
- **Live monitoring** with real-time health gauges, conversation stream, intervention log
- **Tabbed session analysis** — Overview, Timeline, Tools & Prompts, Detections, Hooks & Context
- **Audit trail** — browse immutable events, full-text search, export
- **Settings** — toggle individual observers (12), detectors (16), and strategies (14)

Additional REST API endpoints at `/api/v1/*` for compliance, skills, MCP, analytics, meta, and more.

## 🐍 Python SDK

```python
from forge_sdk import create_harness, quick_run

# One-line run — harness watches a mock agent through 4 turns
result = quick_run("Write a function to validate email addresses")
print(f"Success: {result.success}")
print(f"Detections: {result.detection_count}")
print(f"Interventions: {result.intervention_count}")

# Create a harness for more control
harness = create_harness(preset="claude-code")
result = harness.run("Write a function to validate email addresses")
```

## 🦀 Rust SDK

```rust
use forge_harness::runner;
use forge_sdk::agent::{AgentType, MockAgent};
use forge_sdk::presets::Preset;

let mut agent = MockAgent::new("my-agent", AgentType::Solo);
let result = runner::run_harness_session(
    &mut agent,
    "Implement JWT auth system",
    Preset::Solo,
    None,
).await?;
```

## Why Forge?

| Without Forge | With Forge |
|---|---|
| Agent loops silently | Loop detected → nudge applied → agent breaks out |
| Context hits limit, session fails | Pressure detected at 75% → compacted → never hits limit |
| API key leaks in output | Secret leak detected → circuit break → session stopped |
| Agent hallucinates files | Hallucination detected → nudge → agent corrects |
| Cost spikes 10x | Cost anomaly detected → model swapped → savings |

## Installation

```bash
# Python (Recommended)
pip install forge-agent-sdk

# Rust — build from source
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

## License

MIT
