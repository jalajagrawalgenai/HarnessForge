# Forge — Self-Improving Agent Harness SDK

**The first harness that watches, detects, intervenes, AND improves itself.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-130%20passed-brightgreen.svg)](.)

---

## What Is Forge?

Forge wraps around ANY existing AI agent and provides three capabilities no other tool offers:

| Layer | What It Does | Available Today? |
|---|---|---|
| **Observe + Detect** | 12-dimension real-time watching. 16 issue detectors. | ✅ Yes |
| **Intervene** | 14 autonomous strategies — nudge, compact, rollback, circuit-break... | ❌ **Nobody has this** |
| **Self-Improve** | Mines weakness patterns across sessions. Rewrites its own rules. | ❌ **Nobody has this** |

```
PASSIVE TOOLS:                         FORGE (active):
─────────────────────                  ─────────────────
LangFuse: "Cost spiked at 2:34 PM"     Detects spike at turn 3,
                                       swaps model. Saves $0.12.

LangSmith: "Session failed"            Checkpoints before failure,
                                       rolls back, retries. Succeeds.
```

---

## Quick Start

### Prerequisites

- Rust 1.85+ (`rustc --version`)
- Cargo (bundled with Rust)

### Build & Run

```bash
# Clone
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge

# Run the example (easiest way to see Forge in action)
cargo run -p forge-example-basic-agent

# Run all tests
cargo test --workspace

# Use the CLI
cargo run -p forge-cli -- doctor
cargo run -p forge-cli -- run "Write a function to validate email addresses"
cargo run -p forge-cli -- init --name my-agent --agent-type solo
```

### Example Output

```
╔══════════════════════════════════════════════════════════════╗
║           Forge Harness — Basic Agent Example                ║
╚══════════════════════════════════════════════════════════════╝

🚀 Running agent with Solo preset...
   Task:  Write a function to validate email addresses
   Agent: example-agent

╔══════════════════════════════════════════════════════════════╗
║                    SESSION RESULTS                            ║
╠══════════════════════════════════════════════════════════════╣
║ Agent ID:       example-agent                                ║
║ Success:        ✅ YES                                        ║
║ Observation cycles: 9                                        ║
║ Detections:     0                                            ║
║ Interventions:  0                                            ║
╚══════════════════════════════════════════════════════════════╝
```

---

## SDK Usage

### Wrapping Your Agent

Implement the `AgentAdapter` trait to wrap any existing agent:

```rust
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use forge_sdk::error::ForgeError;
use tokio::sync::mpsc;

struct MyAgent { name: String, model: String }

#[async_trait::async_trait]
impl AgentAdapter for MyAgent {
    fn id(&self) -> String { self.name.clone() }
    fn agent_type(&self) -> AgentType { AgentType::Solo }

    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        mut intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError> {
        // 1. Emit events as your agent works
        event_tx.send(AgentEvent::ThinkingStart {
            agent_id: self.id(), timestamp: chrono::Utc::now(),
        }).await.ok();

        // 2. Call your LLM or agent logic here...
        let response = format!("Working on: {}", task);

        // 3. Emit tool use and token usage
        event_tx.send(AgentEvent::ToolCallStart {
            agent_id: self.id(), tool: "read".into(),
            args: serde_json::json!({"file": "src/main.rs"}),
            timestamp: chrono::Utc::now(),
        }).await.ok();

        event_tx.send(AgentEvent::ToolCallEnd {
            agent_id: self.id(), tool: "read".into(),
            result: ToolResult {
                content: "fn main() { ... }".into(),
                is_error: false, duration_ms: 50, token_count: 100,
            },
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // 4. Check for harness interventions between turns
        while let Ok(intervention) = intervention_rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                _ => { /* handle nudge, compact, pause, etc. */ }
            }
        }

        // 5. Signal completion
        event_tx.send(AgentEvent::Completed {
            agent_id: self.id(),
            summary: "Task complete".into(),
            timestamp: chrono::Utc::now(),
        }).await.ok();

        Ok(AgentOutcome {
            success: true,
            summary: format!("Completed: {}", task),
            output: Some(serde_json::json!({"response": response})),
        })
    }
}
```

### Run Through the Harness

```rust
use forge_sdk::presets::Preset;
use forge_harness::runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut agent = MyAgent { name: "my-agent".into(), model: "claude-sonnet-4-6".into() };

    // Run with full observation + detection + intervention
    let result = runner::run_harness_session(
        &mut agent,
        "Write a function to validate email addresses",
        Preset::Solo,
        None,  // No audit store
    ).await?;

    println!("Observations: {}", result.observation_count);
    println!("Detections:   {}", result.detection_count);
    println!("Interventions: {}", result.intervention_count);
    Ok(())
}
```

### Quick Run (Minimal Setup)

```rust
let mut agent = forge_sdk::agent::MockAgent::new("test", AgentType::Solo);
let result = forge_harness::runner::quick_run(&mut agent, "say hello").await?;
```

### Presets

31 presets for different agent types — each with the right observers, detectors, and strategies:

```rust
// Solo agent (Claude API, ChatGPT, etc.)
runner::run_harness_session(&mut agent, task, Preset::Solo, None).await?;

// Claude Code
runner::run_harness_session(&mut agent, task, Preset::ClaudeCode, None).await?;

// LangGraph
runner::run_harness_session(&mut agent, task, Preset::LangGraph, None).await?;

// CrewAI, AutoGen, LangChain, DSPy, LlamaIndex, Aider, Cline...
// See forge_sdk::presets::Preset for all 31 variants
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    FORGE HARNESS PIPELINE                     │
│                                                               │
│  Agent Events → Observe → Detect → Strategize → Act → Audit  │
│       ↑                                          │            │
│       │        12 observers                      │            │
│       │        16 detectors    14 strategies     ↓            │
│  ┌────┴────┐                          ┌──────────────────┐  │
│  │  AGENT  │←── interventions ────────│    HARNESS       │  │
│  │ ADAPTER │                          │    RUNTIME       │  │
│  │         │─── events ──────────────→│                  │  │
│  └─────────┘                          └──────────────────┘  │
│                                                               │
│  event_tx (agent→harness)     intervention_rx (harness→agent)│
└──────────────────────────────────────────────────────────────┘
```

---

## Packages

| Package | Files | Description |
|---|---|---|
| `forge-sdk` | 19 files | Public API — types, events, AgentAdapter, HarnessBuilder, 31 presets |
| `forge-harness` | 10 files | Pipeline engine — event bus, plugin registry, runtime, checkpoint, human gate, factory |
| `forge-observers` | 14 files | 12-dimensional watchers (token, latency, cost, accuracy, security, reliability, context, orch, comm, compliance, memory, diversity) + health scorer |
| `forge-detectors` | 17 files | 16 detectors (loop, stale, cost_anomaly, deadlock, hallucination, injection, secret_leak, variety, stall, goal_drift, model_mismatch, accuracy_risk, runaway_cost, resource_exhaustion, output_degradation, compliance_gap) |
| `forge-strategies` | 15 files | 14 strategies (nudge, compact, pause, escalate, fork, reroute, rollback, diversify, isolate, circuit_break, replace, interject, degrade, quarantine) |
| `forge-audit` | 16 files | Immutable audit trail, SQLite/Postgres stores, in-memory store, FTS search, hash-chain signing, replay, export, explainer, retention, custody, alerts, SIEM, carbon tracking |
| `forge-meta` | 12 files | Self-improving meta-harness — weakness miner, harness proposer, proposal validator, edit registry, A/B testing, scheduler, cross-model learning |
| `forge-cli` | 1 main + commands | CLI — `forge init`, `forge run`, `forge doctor`, `forge explain`, etc. |
| `forge-server` | routes + ws | axum REST + SSE + WebSocket server |
| `forge-bridge` | 4 files | HTTP client, token counter, model catalog (5 models), cost calculator |
| `forge-mcp` | 4 files | MCP client, server, gateway, discovery |
| `forge-skills` | 3 files | Skill registry, composer, built-in skills |
| `forge-cloud` | 5 files | AWS, Azure, GCP cloud integration traits + deploy |
| **Total** | **180+ source files** | **130 tests, 0 failures** |

---

## CLI Commands

### `forge init` — Scaffold a new project
```bash
cargo run -p forge-cli -- init --name my-agent --agent-type solo
cd my-agent && cargo run
```

### `forge run` — Run agent with harness
```bash
cargo run -p forge-cli -- run "Refactor the auth module"
cargo run -p forge-cli -- run --agent claude-code --preset claude-code "Code review"
cargo run -p forge-cli -- run --dry-run "Test run — observe only"
```

### `forge doctor` — Check system setup
```bash
cargo run -p forge-cli -- doctor
```

### Other commands
```bash
forge explain <session-id>    # Audit report
forge watch <session-id>      # Live TUI (coming soon)
forge bench                   # Benchmark suite
forge improve                 # Meta-harness improvement cycle
forge serve --port 3000       # API server + dashboard
```

---

## 12 Observation Dimensions

| Dimension | What It Watches |
|---|---|
| **Token** | Cache hit rate, dedup, compression, waste tokens |
| **Latency** | p50/p95/p99, TTFT, per-tool timing |
| **Cost** | $/operation, budget burn rate, model efficiency |
| **Accuracy** | Test pass rate, lint errors, verification |
| **Security** | Secret leaks, dangerous tools, prompt injection |
| **Reliability** | Error rate, retry frequency, timeout rate |
| **Context Quality** | Info density, redundancy, staleness |
| **Orchestration** | Agent tree, routing accuracy |
| **Communication** | Message flow, turn fairness, topic coherence |
| **Memory** | Hit rate, knowledge staleness |
| **Compliance** | PII exposure, audit gaps, gate bypass |
| **Diversity** | Approach similarity, solution coverage |

---

## 16 Detectors

| Detector | Triggers On | Severity |
|---|---|---|
| LoopDetector | Same tool 4+ calls, no progress | Warning→Error |
| StaleContextDetector | Same file re-read 3×, pressure >85% | Warning→Error |
| CostAnomalyDetector | Cost >3× moving average | Warning→Error |
| DeadlockDetector | 2+ agents waiting >60s | Error |
| HallucinationDetector | Reference to non-existent file/API | Warning→Error |
| PromptInjectionDetector | "Ignore previous instructions" patterns | Error |
| SecretLeakDetector | API keys, tokens, private keys | **Critical** |
| VarietyCollapseDetector | 3+ agents identical outputs >85% | Warning |
| ConversationStallDetector | No messages for 45s | Warning→Error |
| GoalDriftDetector | Divergence from original task | Warning→Error |
| ModelMismatchDetector | Complex task → weak model | Warning |
| AccuracyRiskDetector | Code generated, no tests | Warning |
| RunawayCostDetector | Cost accelerating (2nd derivative) | Warning→Error |
| ResourceExhaustionDetector | Disk/memory > threshold | Warning→Error |
| OutputDegradationDetector | Quality declining 3+ outputs | Warning→Error |
| ComplianceGapDetector | Human gate skip, audit gap | **Critical** |

---

## 14 Intervention Strategies

| Strategy | Action | Priority |
|---|---|---|
| Nudge | Inject hint into agent context | 10 |
| Compact | Trigger context compression | 20 |
| Diversify | Force agents to use different approaches | 20 |
| Reroute | Change agent's next action (graph) | 22 |
| Escalate | Upgrade model, expand budget | 25 |
| Rollback | Restore from checkpoint, retry | 28 |
| Pause | Pause agent, notify human | 30 |
| Interject | Strong STOP message as user | 35 |
| Quarantine | Route output to sandbox | 40 |
| Replace | Kill agent, spawn replacement | 45 |
| Isolate | Remove dangerous tools | 50 |
| Degrade | Switch to cheaper model | 15 |
| Fork | Split into parallel children | 18 |
| CircuitBreak | Emergency stop ALL agents | 100 |

---

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p forge-harness --lib
cargo test -p forge-observers
cargo test -p forge-strategies
cargo test -p forge-audit
```

---

## Project Status

### ✅ Implemented (Phase 1 Core)
- [x] Full type system: AgentEvent (20 variants), Intervention (17 variants), AgentOutcome
- [x] AgentAdapter trait + MockAgent for testing
- [x] HarnessBuilder + Harness + HarnessRuntime trait
- [x] 12 observers (all with real logic)
- [x] 16 detectors (all with real logic)
- [x] 14 strategies (all with real logic)
- [x] 31 presets (all wireable via factory)
- [x] Pipeline engine (observe→detect→strategize→act→audit)
- [x] Runtime (channel-based agent↔harness communication)
- [x] Plugin registry + factory (preset → concrete types)
- [x] Audit trail (immutable append-only, hash chain, signing)
- [x] SQLite audit store (full implementation)
- [x] In-memory audit store (full implementation)
- [x] Postgres audit store (schema + append)
- [x] Human gate (pause/approve/reject/override state machine)
- [x] Checkpoint manager (save/load/evict)
- [x] CLI: `forge init`, `forge run`, `forge doctor`, `forge explain`
- [x] Meta-harness: weakness miner, proposer, validator, A/B testing
- [x] Audit features: FTS search, replay, export, explainer, retention, custody, alerts, SIEM, carbon
- [x] MCP client/server/gateway modules
- [x] Skills registry + composer
- [x] Cloud traits (AWS, Azure, GCP)
- [x] Working example (`examples/basic-agent/`)
- [x] 130 tests, 0 failures

### 🚧 In Progress (Phase 2)
- [ ] Real AgentAdapter for Claude API (currently MockAgent only)
- [ ] Python bindings (PyO3)
- [ ] TypeScript bindings (NAPI-RS)
- [ ] CLI TUI (`forge watch` with ratatui)
- [ ] Dashboard (Leptos WASM)
- [ ] forge-server route wiring to real SDK
- [ ] Docker image
- [ ] GitHub Actions CI

### 📋 Planned (Phase 3+)
- [ ] VSCode extension
- [ ] Kubernetes operator
- [ ] Plugin marketplace
- [ ] SSO/SAML/OIDC
- [ ] EU AI Act compliance packs
- [ ] LangFuse/W&B native export
- [ ] PagerDuty/OpsGenie integration

---

## License

MIT — see [LICENSE](LICENSE)

---

**Built with Rust 🦀** | 180+ source files | 130 tests | 0 failures
