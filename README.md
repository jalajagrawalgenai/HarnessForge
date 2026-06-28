# Forge — Self-Improving Agent Harness SDK

**The first harness that watches, detects, intervenes, AND improves itself.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.10+-blue.svg)](https://pypi.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-169%20passed-brightgreen.svg)](.)

---

## What Is Forge?

Forge wraps around your AI agent and provides a **4-layer observability pipeline**:

| Layer | What It Does | Status |
|---|---|---|
| **Layer 1: Observe** | 12-dimension real-time watching — token, latency, cost, accuracy, security, reliability, context, orchestration, communication, memory, compliance, diversity | ✅ |
| **Layer 2: Detect** | 16 issue detectors — loop, stale context, cost anomaly, deadlock, hallucination, prompt injection, secret leak, variety collapse, conversation stall, goal drift, model mismatch, accuracy risk, runaway cost, resource exhaustion, output degradation, compliance gap | ✅ |
| **Layer 3: Strategize** | 14 autonomous strategies with priority-based selection — nudge, compact, diversify, reroute, escalate, rollback, pause, interject, quarantine, replace, isolate, degrade, fork, circuit-break | ✅ |
| **Layer 4: Self-Improve** | Meta-harness mines weakness patterns across sessions, proposes rule edits, validates improvements with A/B testing | ✅ |

Every event, hook invocation, tool call, and prompt is traced end-to-end. The analysis API returns the full timeline: what each tool did, which prompt had what latency, what detectors found, which strategy was selected and why.

```
OBSERVE → DETECT → STRATEGIZE → SELF-IMPROVE
   12         16         14             1
observers  detectors  strategies   meta-harness

What Forge catches:
• Agent re-reads same file 4× → StaleContext → Compact context 87%→58%
• Code generated but no tests → AccuracyRisk → Nudge: "Run tests first"
• API key about to leak → SecretLeak → CircuitBreak immediately
• Cost accelerating 2×/turn → RunawayCost → Degrade to cheaper model
```

---

## 🚀 Quick Start — `pip install` + `forge serve`

**One install. Start the dashboard. Everything through the UI.**

```bash
pip install forge-agent-sdk    # Install (builds from source or install wheel)
forge serve                     # Start the dashboard at http://localhost:3000
```

The dashboard is the primary Forge interface. It provides:

- **Run** — type a task, select agent type and preset, click Run. Watch live.
- **Sessions** — browse past sessions with tabbed analysis (Overview, Timeline, Tools & Prompts, Detections, Hooks & Context)
- **Live** — WebSocket-connected real-time view of running sessions
- **Audit** — browse immutable audit events, full-text search, export
- **Settings** — toggle observers (12), detectors (16), strategies (14)

Additional API endpoints are available at `/api/v1/*` for health, compliance, skills, MCP, analytics, meta, and more.

---

## Python API

```python
from forge_sdk import create_harness, quick_run, list_presets, list_detectors

# See what's available
print("Presets:", list_presets())      # 31 presets
print("Detectors:", list_detectors())  # 16 detectors

# One-line run — harness watches a mock agent
result = quick_run("Write a function to validate email addresses")
print(f"Success: {result.success}")
print(f"Observations: {result.observation_count}")
print(f"Detections: {result.detection_count}")
print(f"Interventions: {result.intervention_count}")

# Create a harness for more control
harness = create_harness(preset="solo")
result = harness.run("Build a REST API for a todo app")

# Dry run — observe and detect only (no intervention)
result = harness.dry_run("Test run — observe only")
```

### Python API Reference

| Function | Description |
|---|---|
| `quick_run(task, preset="solo", turns=4)` | One-shot: create harness, run task, return result |
| `create_harness(preset="solo")` | Create a `PyHarness` for repeated use |
| `harness.run(task)` | Run task through the full observe → detect → intervene pipeline |
| `harness.dry_run(task)` | Observe and detect only (no intervention) |
| `harness.run_with(task, preset, turns)` | Run with custom preset and turn count |
| `list_presets()` | List all 31 available presets |
| `list_detectors()` | List all 16 detectors |
| `list_strategies()` | List all 14 intervention strategies |
| `list_observers()` | List all 12 observation dimensions |
| `get_version()` | Get the forge-agent-sdk version string |

### HarnessRunResult Fields

| Field | Type | Description |
|---|---|---|
| `agent_id` | `str` | Agent identifier |
| `success` | `bool` | Whether the task completed successfully |
| `observation_count` | `int` | Number of pipeline observation cycles |
| `detection_count` | `int` | Issues detected |
| `intervention_count` | `int` | Interventions applied |
| `to_dict()` | `dict` | Convert result to a Python dictionary |

### Build from Source (Python)

```bash
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge/packages/forge-py
pip install maturin
maturin develop

# Verify
python -c "from forge_sdk import get_version; print(get_version())"
```

---

## Rust — Build from Source

```bash
# Prerequisites: Rust 1.85+
git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

### Run the Example

```bash
cargo run -p forge-example-basic-agent
```

### Use the Rust CLI

```bash
# Start the dashboard server
cargo run -p forge-cli -- serve

# Run an agent through the harness
cargo run -p forge-cli -- run "Write a function to validate email addresses"

# Run with a specific preset
cargo run -p forge-cli -- run --preset claude-code "Code review this PR"

# Dry run — observe only, no intervention
cargo run -p forge-cli -- run --dry-run "Test the harness config"

# Scaffold a new Forge project
cargo run -p forge-cli -- init --name my-agent

# Check system dependencies
cargo run -p forge-cli -- doctor

# Get an audit report
cargo run -p forge-cli -- explain <session-id>
```

### Use the SDK in Your Own Rust Project

```toml
[dependencies]
forge-sdk = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
forge-harness = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

```rust
use forge_harness::runner;
use forge_sdk::agent::{AgentType, MockAgent};
use forge_sdk::presets::Preset;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut agent = MockAgent::new("my-agent", AgentType::Solo)
        .with_turns(4)
        .with_success(true);

    let result = runner::run_harness_session(
        &mut agent,
        "Write a function to validate email addresses",
        Preset::Solo,
        None,
    ).await?;

    println!("Agent:         {}", result.agent_id);
    println!("Success:       {}", result.success);
    println!("Observations:  {}", result.observation_count);
    println!("Detections:    {}", result.detection_count);
    println!("Interventions: {}", result.intervention_count);
    Ok(())
}
```

### Wrapping a Real Agent (AgentAdapter Trait)

Implement `AgentAdapter` to wrap any agent (Claude API, LangGraph, CrewAI, etc.):

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
        // Signal start
        event_tx.send(AgentEvent::Started {
            agent_id: self.id(), task: task.to_string(),
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // Call your LLM
        let response = your_llm_call(task).await;

        // Report tool usage
        event_tx.send(AgentEvent::ToolCallEnd {
            agent_id: self.id(), tool: "llm_call".into(),
            result: ToolResult {
                content: response.clone(),
                is_error: false, duration_ms: 1200, token_count: 500,
            },
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // Check for harness interventions
        while let Ok(intervention) = intervention_rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                Intervention::Nudge { message, .. } => {
                    eprintln!("💡 Harness nudge: {}", message);
                }
                Intervention::Compact { target_ratio, .. } => {
                    eprintln!("📦 Compacting context to {:.0}%", target_ratio * 100.0);
                }
                _ => {}
            }
        }

        // Signal completion
        event_tx.send(AgentEvent::Completed {
            agent_id: self.id(),
            summary: "Task completed".into(),
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

---

## Supported Agent Types

Forge has 31 presets. Each preset configures which observers, detectors, and strategies are active for that agent type:

| Preset | Agent Type | Preset | Agent Type |
|---|---|---|---|
| `solo` | Single LLM agent | `langgraph` | LangGraph agent |
| `crewai` | CrewAI multi-agent | `autogen` | AutoGen multi-agent |
| `langchain` | LangChain agent | `openai-swarm` | OpenAI Swarm |
| `semantic-kernel` | Semantic Kernel | `haystack` | Haystack by deepset |
| `dspy` | DSPy optimizer | `llamaindex` | LlamaIndex agent |
| `taskweaver` | TaskWeaver | `agno` | Agno framework |
| `atomic-agents` | Atomic Agents | `bee-agent` | Bee Agent Framework |
| `pydantic-ai` | PydanticAI | `claude-code` | Claude Code CLI |
| `aider` | Aider CLI | `cline` | Cline (VS Code) |
| `continue` | Continue.dev | `vercel-ai` | Vercel AI SDK |
| `copilot` | GitHub Copilot | `cursor` | Cursor IDE |
| `windsurf` | Windsurf IDE | `devin` | Devin (Cognition) |
| `amazon-q` | Amazon Q Developer | `replit-agent` | Replit Agent |
| `pearai` | PearAI | `bolt-new` | Bolt.new |
| `lovable` | Lovable | `v0` | v0 by Vercel |
| `custom` | Custom agent | | |

Use them the same way:

```rust
// Rust
runner::run_harness_session(&mut agent, task, Preset::ClaudeCode, None).await?;
```

```python
# Python
harness = create_harness(preset="claude-code")
result = quick_run("task", preset="crewai")
```

---

## Architecture — 4-Layer Pipeline

```
┌──────────────────────────────────────────────────────────────────┐
│                    FORGE HARNESS — 4 LAYER PIPELINE                │
│                                                                   │
│  Agent Events                                                     │
│      │                                                            │
│      ├─→ [Layer 1: OBSERVE]                                       │
│      │    event_to_observation() — flattens raw event into        │
│      │    JSON with ALL fields detectors need                     │
│      │    12 observers: token, latency, cost, accuracy,           │
│      │    security, reliability, context, orch, comm,             │
│      │    memory, compliance, diversity                           │
│      │                                                            │
│      ├─→ [Layer 2: DETECT]                                        │
│      │    detect_from_events() — scans event history for          │
│      │    patterns: loops (deduped per tool), stale context,      │
│      │    cost anomalies, secret leaks, accuracy risk...          │
│      │    16 detectors with category-level deduplication          │
│      │                                                            │
│      ├─→ [Layer 3: STRATEGIZE]                                    │
│      │    Try ALL 14 strategies against each detection.           │
│      │    Pick highest-priority match (nudge=10, compact=20,      │
│      │    circuit_break=100). Not first-match — best-match.       │
│      │    14 strategies with priority-based selection             │
│      │                                                            │
│      └─→ [Layer 4: SELF-IMPROVE]                                  │
│           POST /v1/meta/improve — mines weakness patterns         │
│           across completed sessions. Proposes rule edits.         │
│           GET /v1/meta/weaknesses — current weakness patterns     │
│           GET /v1/meta/edits — pending harness rule changes       │
│                                                                   │
│  ┌──────────┐                          ┌──────────────────┐      │
│  │  AGENT   │←── interventions ────────│  FORGE SERVER    │      │
│  │ (Claude, │    nudge, compact,       │  (Axum + WS)     │      │
│  │  Cursor, │    circuit_break...      │  17 route modules│      │
│  │  CrewAI) │─── events via hooks ────→│  REST + SSE + WS │      │
│  └──────────┘                          └──────────────────┘      │
│                                                                   │
│  Persistence: ~/.forge/sessions/*.json — survives restarts        │
│  Hooks: ~/.claude/settings.json — auto-registered for Claude Code │
└──────────────────────────────────────────────────────────────────┘
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
| Degrade | Switch to cheaper model | 15 |
| Fork | Split into parallel children | 18 |
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
| CircuitBreak | Emergency stop ALL agents | 100 |

---

## Packages

| Package | Files | Description |
|---|---|---|
| `forge-sdk` | 19 | Public API — types, events, AgentAdapter, HarnessBuilder, 31 presets |
| `forge-harness` | 10 | Pipeline engine — event bus, plugin registry, runtime, checkpoint, human gate |
| `forge-observers` | 14 | 12-dimensional watchers + health scorer |
| `forge-detectors` | 17 | 16 detectors with category-level deduplication |
| `forge-strategies` | 15 | 14 intervention strategies with priority-based selection |
| `forge-audit` | 16 | Immutable audit trail, SQLite/Postgres stores, FTS, hash-chain, replay, export |
| `forge-meta` | 12 | Self-improving meta-harness — weakness miner, proposer, validator, A/B testing |
| `forge-cli` | 8 | CLI — `forge init`, `forge run`, `forge serve`, `forge doctor`, etc. |
| `forge-server` | 17 routes | Axum REST + SSE + WebSocket server — full pipeline per ingest, tabbed analysis |
| `forge-bridge` | 4 | HTTP client, token counter, model catalog, cost calculator |
| `forge-mcp` | 4 | MCP client, server, gateway, discovery |
| `forge-skills` | 3 | Skill registry, composer, built-in skills |
| `forge-py` | 3 | Python bindings (PyO3) — `pip install forge-agent-sdk` |
| `forge-adapters` | 6 | AgentAdapter implementations for CLI, HTTP, Python, Bridge agents + AdapterFactory |
| `forge-cloud` | 6 | AWS, Azure, GCP cloud integration traits + deploy |
| `forge-auth` | 1 | SSO/OIDC authentication traits (Okta, Azure AD, Google Workspace) |
| `forge-compliance` | 1 | Compliance report generation (EU AI Act, SOC 2, GDPR, HIPAA, PCI DSS) |
| `forge-export` | 1 | Export targets (LangFuse, OpenTelemetry, PagerDuty, Slack, etc.) |
| `forge-marketplace` | 1 | Plugin marketplace client (browse, install, publish) |
| `forge-vscode` | 6 | VS Code extension — sidebar, TreeViews, 6 commands |
| `forge-dashboard` | 1 | Dashboard crate |
| `forge-integrations` | 1 | Integration adapters |
| **Total** | **~188 source files** | **169 tests, 0 failures** |

---

## Project Status

### ✅ Core Pipeline (Implemented)
- [x] 4-layer pipeline: Observe → Detect → Strategize → Self-Improve
- [x] 12 observers, 16 detectors, 14 strategies (all with real logic)
- [x] `event_to_observation()` — bridges observer↔detector field mismatch
- [x] Priority-based strategy selection (try all, pick highest-priority match)
- [x] Category-level detector deduplication + per-tool loop dedup
- [x] forge-server: 17 route modules, REST + SSE + WebSocket API
- [x] Tabbed session analysis: Overview, Timeline, Tools & Prompts, Detections, Hooks & Context
- [x] Token + cost estimation with estimated/real labeling
- [x] Model auto-detection from hooks + environment variables
- [x] Session persistence to `~/.forge/sessions/*.json` (survives restarts)
- [x] AgentAdapter trait + MockAgent + HarnessBuilder + HarnessRuntime
- [x] 31 presets (configuration templates for different agent types)
- [x] Audit trail (immutable append-only, hash chain, signing)
- [x] SQLite, In-memory, and Postgres audit stores
- [x] Human gate (pause/approve/reject/override state machine)
- [x] Checkpoint manager (save/load/evict)
- [x] Meta-harness: weakness miner, proposer, validator, A/B testing
- [x] Python bindings — `pip install forge-agent-sdk`
- [x] Hook auto-registration for Claude Code (`~/.claude/settings.json`)
- [x] CI/CD: `cargo test`, `cargo fmt`, `cargo clippy` all passing

### 🚧 In Progress
- [ ] Dashboard UI pages beyond Run, Sessions, Live, Audit, Settings (remaining routes have API but thin UI)
- [ ] TypeScript bindings (NAPI-RS)
- [ ] CLI TUI (`forge watch` with ratatui)
- [ ] Plugin marketplace registry server (client crate exists)
- [ ] Real-agent AdapterFactory wiring for all 31 presets (traits + stubs exist)

### 📋 Planned
- [ ] Docker Compose quickstart
- [ ] GitHub Action for CI (`forgelabs/forge-action`)
- [ ] Performance profiler (`forge profile`)
- [ ] Interactive debugger (`forge debug`)
- [ ] Carbon footprint tracking

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

## Using Forge with Claude Code

Forge auto-registers hooks in `~/.claude/settings.json` on first import. Every Claude Code session is then observed automatically:

```
┌─────────────────────────────────────────────────────────────┐
│                    FORGE HARNESS                              │
│                                                               │
│  Claude Code ──→ Forge watches ──→ Detects issues ──→ Fixes │
│       ↑                                            │          │
│       │        12 observers watch                   │          │
│       │        16 detectors scan     14 strategies  ↓          │
│       │                                            │          │
│       └──────── interventions (nudge, compact...) ─┘          │
└─────────────────────────────────────────────────────────────┘
```

### Quick Test

```bash
pip install forge-agent-sdk
forge serve
# Open http://localhost:3000 → any Claude Code session appears automatically
```

### What Forge Catches

- **Re-reading the same file** → StaleContext detection → auto-compacts context
- **Getting stuck in a loop** → Loop detection → nudges to break the cycle
- **Skipping tests** → AccuracyRisk detection → nudges to run tests
- **Context filling up** → Context pressure detection → compresses before overflow
- **Accidental secret leak** → Secret leak detection → circuit breaks immediately

---

## License

MIT — see [LICENSE](LICENSE)

---

**Built with Rust 🦀** | ~188 source files | 169 tests | 0 failures | 4-layer pipeline (Observe → Detect → Strategize → Self-Improve)
