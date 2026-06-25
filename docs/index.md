# Forge — The Agent Harness SDK

**Self-improving harness that watches, detects, and intervenes for ANY AI agent.**

Forge wraps around your existing agents (LangGraph, CrewAI, AutoGen, raw Claude API) and adds:

- **12-dimensional observation** — token, latency, cost, accuracy, security, and more
- **16 real-time detectors** — loop, stale context, secret leak, hallucination, deadlock...
- **14 autonomous interventions** — nudge, compact, pause, escalate, circuit break...
- **Immutable audit trail** — hash-chain integrity, full-text search, session replay
- **Self-improving meta-harness** — mines weaknesses, proposes edits, validates improvements

## Quick Start

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
cargo install forge-sdk
```

## License

MIT
