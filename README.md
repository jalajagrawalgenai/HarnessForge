# HarnessForge — Self-Improving Agent Harness SDK

**The first harness that watches, detects, intervenes, AND improves itself.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-68%20passed-brightgreen.svg)](.)

Forge wraps around ANY existing AI agent (LangGraph, CrewAI, AutoGen, Claude API) and adds the missing layer that no observability tool provides: **autonomous intervention and self-improvement.**

## Why Forge?

| Without Forge | With Forge |
|---|---|
| Agent loops silently → session fails | Loop detected at turn 4 → nudge applied → agent breaks out |
| Context hits limit → crash | Pressure detected at 75% → compacted → never hits limit |
| API key leaks in output → disaster | Secret leak detected → circuit break → session stopped |
| Agent hallucinates files → bugs | Hallucination detected → nudge → agent corrects |
| Cost spikes 10x → budget blown | Cost anomaly detected → model swapped → savings |

## Architecture

```
YOUR AGENT (LangGraph / CrewAI / Claude API / Custom)
       │
       │ event stream
       ↓
┌──────────────────────────────────────────┐
│            FORGE HARNESS                  │
│                                          │
│  Observe → Detect → Strategize → Act → Audit │
│  (12 dims)  (16 dets)  (14 strats)        │
│                                          │
│  ┌──────────────────────────────────┐    │
│  │      META-HARNESS (Level 2)      │    │
│  │  Mine weaknesses → Propose edits │    │
│  │  → Regression test → Apply      │    │
│  │  Self-improves over time         │    │
│  └──────────────────────────────────┘    │
└──────────────────────────────────────────┘
```

## Quick Start

```rust
use forge_sdk::prelude::*;

// 1. Your existing agent
let my_agent = MyAgent::new();

// 2. Wrap in Forge harness
let harness = Harness::builder()
    .observe(all_observers!())
    .detect(all_detectors!())
    .strategize(all_strategies!())
    .audit(AuditConfig::default())
    .build()?;

// 3. Run — Forge watches, detects, intervenes
let session = harness.run(&mut my_agent, "Build JWT auth").await?;
println!("{}", session.audit_report());
```

## Packages

| Package | Description |
|---|---|
| `forge-sdk` | Public API — types, traits, HarnessBuilder |
| `forge-harness` | Pipeline engine — event bus, runtime, checkpoint |
| `forge-observers` | 12 watchers — token, latency, cost, accuracy, security... |
| `forge-detectors` | 16 detectors — loop, secret leak, hallucination, deadlock... |
| `forge-strategies` | 14 interventions — nudge, compact, circuit break... |
| `forge-audit` | Immutable trail, hash-chain integrity, SQLite, replay |
| `forge-meta` | Self-improving meta-harness — mine, propose, validate |
| `forge-bridge` | Model catalog, cost calculator, LiteLLM client |
| `forge-mcp` | MCP client, server, gateway |
| `forge-skills` | Skill registry, composer, built-in skills |
| `forge-cloud` | AWS, Azure, GCP integrations |
| `forge-cli` | Full CLI — 16 commands |

## Installation

```bash
cargo install forge-sdk
```

## CLI

```bash
forge init          # Scaffold new project
forge run "task"    # Run agent with harness
forge watch         # Live TUI dashboard
forge explain <id>  # Human-readable audit report
forge improve       # Run self-improvement cycle
forge bench         # Benchmark vs Claude Code
```

## Documentation

Full docs: `docs/` or [mkdocs](mkdocs.yml)

## License

MIT

---

Built with Rust 🦀
