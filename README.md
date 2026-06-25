# Forge — Self-Improving Agent Harness SDK

**The first harness that watches, detects, intervenes, AND improves itself.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.10+-blue.svg)](https://pypi.org)
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

### Python Quick Start

```bash
# Install (once published to PyPI)
pip install forge-agent-sdk

# Or from source
cd packages/forge-py
pip install maturin
maturin develop
```

```python
from forge_sdk import create_harness, quick_run, list_presets, list_detectors

# See what's available
for p in list_presets():
    print(p)

# One-line run — harness watches a mock agent
result = quick_run("Write a function to validate email addresses")
print(f"Success: {result.success}")
print(f"Detections: {result.detection_count}")
print(f"Interventions: {result.intervention_count}")

# Use a specific preset with more turns
result = quick_run("Build a REST API", preset="claude-code", turns=8)

# Harness object API — configure once, run many times
harness = create_harness(preset="solo")
result = harness.run("Add JWT authentication")
result = harness.dry_run("Test run — observe only, no intervention")
print(result.to_dict())
```

---

## Beginner's Guide: Wrap Claude Code with Forge

This step-by-step guide shows how to wrap **Claude Code** (or any AI agent) inside the Forge harness. You'll build a real agent adapter that sends events to the harness and checks for interventions.

### Step 1: Clone & Build

```bash
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

### Step 2: Run the Built-in Example

```bash
cargo run -p forge-example-basic-agent
```

You'll see:
```
╔══════════════════════════════════════════════════════════════╗
║           Forge Harness — Basic Agent Example                ║
╚══════════════════════════════════════════════════════════════╝

🚀 Running agent with Solo preset...
   Task:  Write a function to validate email addresses

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

### Step 3: Scaffold Your Own Agent Project

```bash
# Create a new project from the Forge template
cargo run -p forge-cli -- init --name my-claude-agent --agent-type claude-code
cd my-claude-agent
```

This creates:
```
my-claude-agent/
├── Cargo.toml          # Depends on forge-sdk + forge-harness
├── forge.toml          # Harness configuration
├── src/
│   └── main.rs         # AgentAdapter template
└── .gitignore
```

### Step 4: Implement the Claude Code Adapter

Edit `src/main.rs` to wrap Claude Code. The key idea: Claude Code runs as a CLI process. Your adapter spawns it, streams its output as AgentEvents to the harness, and checks for interventions:

```rust
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use forge_sdk::error::ForgeError;
use forge_harness::runner;
use tokio::sync::mpsc;
use tokio::process::Command;
use std::time::Instant;

struct ClaudeCodeAgent {
    id: String,
    work_dir: String,
}

#[async_trait::async_trait]
impl AgentAdapter for ClaudeCodeAgent {
    fn id(&self) -> String { self.id.clone() }
    fn agent_type(&self) -> AgentType { AgentType::ClaudeCode }

    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        mut intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError> {
        let now = chrono::Utc::now();

        // 1. Tell the harness: agent is starting
        event_tx.send(AgentEvent::Started {
            agent_id: self.id(), task: task.to_string(), timestamp: now,
        }).await.ok();

        // 2. Spawn Claude Code as a subprocess
        event_tx.send(AgentEvent::ThinkingStart {
            agent_id: self.id(), timestamp: chrono::Utc::now(),
        }).await.ok();

        let start = Instant::now();
        let output = Command::new("claude")
            .arg("-p")           // print mode (non-interactive)
            .arg(task)           // the task
            .arg("--output-format") .arg("stream-json")  // stream events
            .current_dir(&self.work_dir)
            .output()
            .await
            .map_err(|e| ForgeError::AgentFailed {
                reason: format!("Claude Code failed: {}", e),
            })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let duration_ms = start.elapsed().as_millis() as u64;

        // 3. Tell the harness about the tool call (Claude Code = a tool)
        event_tx.send(AgentEvent::ToolCallEnd {
            agent_id: self.id(),
            tool: "claude-code".into(),
            result: ToolResult {
                content: stdout.to_string(),
                is_error: !output.status.success(),
                duration_ms,
                token_count: stdout.len() as u64 / 4, // rough estimate
            },
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // 4. Check for harness interventions
        //    The harness may inject a nudge like:
        //    "Note: file X was already read. Proceed without re-reading."
        while let Ok(intervention) = intervention_rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                Intervention::Nudge { message, .. } => {
                    eprintln!("💡 Harness nudge: {}", message);
                }
                Intervention::Compact { target_ratio, .. } => {
                    eprintln!("📦 Harness compact to {:.0}%", target_ratio * 100.0);
                }
                _ => {}
            }
        }

        // 5. Signal completion
        event_tx.send(AgentEvent::Completed {
            agent_id: self.id(),
            summary: stdout.chars().take(200).collect(),
            timestamp: chrono::Utc::now(),
        }).await.ok();

        Ok(AgentOutcome {
            success: output.status.success(),
            summary: format!("Claude Code completed in {}ms", duration_ms),
            output: Some(serde_json::json!({"stdout": stdout})),
        })
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut agent = ClaudeCodeAgent {
        id: "claude-code-1".into(),
        work_dir: ".".into(),
    };

    println!("🚀 Running Claude Code with Forge harness...");
    let result = runner::run_harness_session(
        &mut agent,
        "Write a Rust function to validate email addresses",
        forge_sdk::presets::Preset::ClaudeCode,
        None,
    ).await?;

    println!("✅ Done!");
    println!("   Observations:  {}", result.observation_count);
    println!("   Detections:    {}", result.detection_count);
    println!("   Interventions: {}", result.intervention_count);
    Ok(())
}
```

### Step 5: Run Your Wrapped Agent

```bash
cargo run
```

Forge will watch Claude Code in real-time, detect issues (loops, stale context, cost anomalies), and intervene if needed.

### Step 6: See What Happened

```bash
cargo run -p forge-cli -- explain <session-id>
```

This prints a human-readable audit report showing every detection and intervention.

### Using Other Agents

The same pattern works for any agent:

| Agent | Spawn Command | AgentType Preset |
|---|---|---|
| **Claude Code** | `claude -p "task"` | `ClaudeCode` |
| **Aider** | `aider --message "task"` | `Aider` |
| **OpenAI API** | HTTP POST to `/v1/chat/completions` | `Solo` |
| **LangGraph** | Python subprocess | `LangGraph` |
| **Custom script** | Any CLI command | `Solo` or `Custom` |

---

## Publishing to PyPI

### Build the Python Package

```bash
cd packages/forge-py

# Install build tools
pip install maturin twine

# Build the wheel (compiles Rust → .pyd)
maturin build --release

# Output: target/wheels/forge_sdk-0.1.0-cp312-cp312-win_amd64.whl
```

### Test Locally

```bash
# Install in development mode
maturin develop

# Test it
python -c "from forge_sdk import quick_run; r = quick_run('hello'); print(r.success)"
```

### Upload to PyPI

```bash
# First, set your PyPI token
export TWINE_USERNAME=__token__
export TWINE_PASSWORD=pypi-your-token-here

# Upload
twine upload target/wheels/forge_sdk-*.whl
```

### Automated CI Release

Add to `.github/workflows/release.yml`:

```yaml
name: Release to PyPI
on:
  push:
    tags: ['v*']

jobs:
  publish:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: PyO3/maturin-action@v1
        with:
          command: build
          args: --release --out dist
      - uses: pypa/gh-action-pypi-publish@release/v1
        with:
          password: ${{ secrets.PYPI_API_TOKEN }}
```

### Build for Multiple Python Versions

```bash
# Build for specific Python version
maturin build --release -i python3.10
maturin build --release -i python3.11
maturin build --release -i python3.12

# Or use manylinux Docker for Linux builds
docker run --rm -v $(pwd):/io ghcr.io/pyo3/maturin build --release
```

### After Publishing

Users install with:
```bash
pip install forge-agent-sdk
```

And use immediately:
```python
from forge_sdk import quick_run
print(quick_run("Say hello").success)
```

---

## SDK Usage

### Wrapping Your Agent (Rust)

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
| `forge-py` | 3 files | Python bindings (PyO3) — `pip install forge-agent-sdk` |
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
- [x] Python bindings (`pip install forge-agent-sdk`)
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
