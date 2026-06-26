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

# Python — Quick Start

### Step 1: Install

```bash
pip install forge-agent-sdk
```

Requirements: Python 3.10+ (including 3.14). Works on Windows, macOS, and Linux.

### Step 2: Run Your First Harnessed Agent

```python
from forge_sdk import quick_run, list_presets, list_detectors

# See what's available
print("Presets:", list_presets())
print("Detectors:", list_detectors())

# One-line run — harness watches a mock agent through 4 turns
result = quick_run("Write a function to validate email addresses")
print(f"Success:       {result.success}")
print(f"Observations:  {result.observation_count}")
print(f"Detections:    {result.detection_count}")
print(f"Interventions: {result.intervention_count}")
```

### Step 3: Use a Real Harness (More Control)

```python
from forge_sdk import create_harness

# Create a harness with the "solo" preset (full observer + detector + strategy suite)
harness = create_harness(preset="solo")

# Run a task
result = harness.run("Build a REST API for a todo app")
print(result.to_dict())

# Dry run — observe and detect, but don't intervene (safe for testing)
result = harness.dry_run("Test run — observe only, no intervention")
print(f"Would have intervened: {result.intervention_count} times")

# Custom preset with more turns
result = harness.run_with("Add JWT authentication", preset="claude-code", turns=8)
```

### Step 4: Expected Output

```
Presets: ['solo', 'langgraph', 'crewai', 'autogen', 'langchain', 'openai-swarm',
          'semantic-kernel', 'haystack', 'dspy', 'llamaindex', 'taskweaver',
          'agno', 'atomic-agents', 'bee-agent', 'pydantic-ai', 'claude-code',
          'aider', 'cline', 'continue', 'vercel-ai', 'copilot', 'cursor',
          'windsurf', 'devin', 'amazon-q', 'replit-agent', 'pearai',
          'bolt-new', 'lovable', 'v0', 'custom']  # 31 total
Detectors: ['loop', 'stale_context', 'cost_anomaly', 'deadlock', 'hallucination',
            'prompt_injection', 'secret_leak', 'variety_collapse', 'conversation_stall',
            'goal_drift', 'model_mismatch', 'accuracy_risk', 'runaway_cost',
            'resource_exhaustion', 'output_degradation', 'compliance_gap']  # 16 total

Success:       True
Observations:  9
Detections:    0
Interventions: 0
```

When the harness DOES detect an issue (e.g., agent re-reads the same file 4 times):

```
⚠  HARNESS [T6]: StaleContext detected. Context pressure 87%.
   → Strategy: Compact. Context reduced 87% → 58%. Saved 4.2K tokens.

🔧 HARNESS [T9]: AccuracyRisk detected. Code generated but no tests run.
   → Strategy: Nudge. "Run the tests before proceeding."
```

### All Python API Functions

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

# Rust — Quick Start

### Step 1: Clone and Build

```bash
# Prerequisites: Rust 1.85+
rustc --version

git clone https://github.com/jalajagrawalgenai/HarnessForge.git
cd HarnessForge
cargo build --release
```

### Step 2: Run the Built-in Example

```bash
cargo run -p forge-example-basic-agent
```

### Step 3: Expected Output

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

✅ No issues detected — agent ran cleanly.

── Try it yourself ──────────────────────────────────────────
  forge init --name my-agent --agent-type solo
  cd my-agent && cargo run
  forge run "Your task here"
```

### Step 4: Use the SDK in Your Own Rust Project

Add Forge to your `Cargo.toml`:

```toml
[dependencies]
forge-sdk = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
forge-harness = { git = "https://github.com/jalajagrawalgenai/HarnessForge.git" }
tokio = { version = "1", features = ["full"] }
anyhow = "1"
```

Then in `src/main.rs`:

```rust
use forge_harness::runner;
use forge_sdk::agent::{AgentType, MockAgent};
use forge_sdk::presets::Preset;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a mock agent (in production, implement AgentAdapter for your real agent)
    let mut agent = MockAgent::new("my-agent", AgentType::Solo)
        .with_turns(4)
        .with_success(true);

    // Run through the Forge harness
    let result = runner::run_harness_session(
        &mut agent,
        "Write a function to validate email addresses",
        Preset::Solo,
        None, // No audit store (use Some(store) for persistence)
    ).await?;

    // See what happened
    println!("Agent:         {}", result.agent_id);
    println!("Success:       {}", result.success);
    println!("Observations:  {}", result.observation_count);
    println!("Detections:    {}", result.detection_count);
    println!("Interventions: {}", result.intervention_count);

    Ok(())
}
```

### Step 5: Run It

```bash
cargo run
```

### Step 6: Use the CLI

```bash
# Scaffold a new Forge project
cargo run -p forge-cli -- init --name my-agent --agent-type solo
cd my-agent && cargo run

# Run a task through the harness
cargo run -p forge-cli -- run "Refactor the auth module to use JWT"

# Run with a specific preset
cargo run -p forge-cli -- run --preset claude-code "Code review this PR"

# Dry run — observe only, no intervention
cargo run -p forge-cli -- run --dry-run "Test the harness config"

# Check your system setup
cargo run -p forge-cli -- doctor

# Get an audit report for a session
cargo run -p forge-cli -- explain <session-id>
```

### Wrapping a Real Agent (Rust — AgentAdapter Trait)

Implement the `AgentAdapter` trait to wrap any existing agent (Claude API, LangGraph, CrewAI, etc.):

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
        // 1. Signal: agent is starting
        event_tx.send(AgentEvent::Started {
            agent_id: self.id(), task: task.to_string(),
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // 2. Signal: agent is thinking
        event_tx.send(AgentEvent::ThinkingStart {
            agent_id: self.id(), timestamp: chrono::Utc::now(),
        }).await.ok();

        // 3. Call your LLM or agent logic here...
        let response = your_llm_call(task).await;

        // 4. Report tool usage
        event_tx.send(AgentEvent::ToolCallEnd {
            agent_id: self.id(), tool: "llm_call".into(),
            result: ToolResult {
                content: response.clone(),
                is_error: false, duration_ms: 1200, token_count: 500,
            },
            timestamp: chrono::Utc::now(),
        }).await.ok();

        // 5. Check for harness interventions
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

        // 6. Signal completion
        event_tx.send(AgentEvent::Completed {
            agent_id: self.id(),
            summary: "Task completed successfully".into(),
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

Then run it:

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut agent = MyAgent { name: "claude".into(), model: "claude-sonnet-4-6".into() };

    let result = runner::run_harness_session(
        &mut agent,
        "Write a function to validate email addresses",
        Preset::Solo,
        None,
    ).await?;

    println!("Observations:  {}", result.observation_count);
    println!("Detections:    {}", result.detection_count);
    println!("Interventions: {}", result.intervention_count);
    Ok(())
}
```

---

## Supported Agent Types (31 Presets)

| Preset | Agent Type | Preset | Agent Type |
|---|---|---|---|
| `solo` | Single LLM agent (Claude API, ChatGPT) | `langgraph` | LangGraph agent |
| `crewai` | CrewAI multi-agent | `autogen` | AutoGen multi-agent |
| `langchain` | LangChain agent | `openai-swarm` | OpenAI Swarm |
| `semantic-kernel` | Microsoft Semantic Kernel | `haystack` | Haystack by deepset |
| `dspy` | DSPy optimizer | `llamaindex` | LlamaIndex agent |
| `taskweaver` | Microsoft TaskWeaver | `agno` | Agno framework |
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

Use them all the same way:

```rust
// Rust
runner::run_harness_session(&mut agent, task, Preset::ClaudeCode, None).await?;
runner::run_harness_session(&mut agent, task, Preset::LangGraph, None).await?;
```

```python
# Python
harness = create_harness(preset="claude-code")
harness = create_harness(preset="langgraph")
result = quick_run("task", preset="crewai")
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

## Packages

| Package | Files | Description |
|---|---|---|
| `forge-sdk` | 19 files | Public API — types, events, AgentAdapter, HarnessBuilder, 31 presets |
| `forge-harness` | 10 files | Pipeline engine — event bus, plugin registry, runtime, checkpoint, human gate, factory |
| `forge-observers` | 14 files | 12-dimensional watchers + health scorer |
| `forge-detectors` | 17 files | 16 detectors |
| `forge-strategies` | 15 files | 14 intervention strategies |
| `forge-audit` | 16 files | Immutable audit trail, SQLite/Postgres stores, FTS, hash-chain, replay, export |
| `forge-meta` | 12 files | Self-improving meta-harness — weakness miner, proposer, validator, A/B testing |
| `forge-cli` | 1 main + commands | CLI — `forge init`, `forge run`, `forge doctor`, `forge explain`, etc. |
| `forge-server` | routes + ws | axum REST + SSE + WebSocket server |
| `forge-bridge` | 4 files | HTTP client, token counter, model catalog, cost calculator |
| `forge-mcp` | 4 files | MCP client, server, gateway, discovery |
| `forge-skills` | 3 files | Skill registry, composer, built-in skills |
| `forge-py` | 3 files | Python bindings (PyO3) — `pip install forge-agent-sdk` |
| `forge-adapters` | 6 files | Real AgentAdapter impls for ALL 31 agent types (CLI, HTTP, Python, Bridge) + AdapterFactory |
| `forge-cloud` | 5 files | AWS, Azure, GCP cloud integration traits + deploy |
| **Total** | **180+ source files** | **130 tests, 0 failures** |

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

## Publishing to PyPI

### Automated CI (Push a Tag)

The `.github/workflows/publish-pypi.yml` workflow builds wheels for:
- **3 OSes:** Ubuntu, Windows, macOS
- **5 Python versions:** 3.10, 3.11, 3.12, 3.13, 3.14

```bash
# Bump version in both files, then:
git add -A && git commit -m "Release v0.1.4"
git tag v0.1.4
git push origin main && git push origin v0.1.4
```

CI builds 15 wheels and publishes to PyPI automatically.

### Manual Build

```bash
cd packages/forge-py
pip install maturin twine
maturin build --release
twine upload target/wheels/forge_sdk-*.whl
```

---

## Project Status

### ✅ Implemented
- [x] Full type system: AgentEvent (20 variants), Intervention (17 variants), AgentOutcome
- [x] AgentAdapter trait + MockAgent
- [x] HarnessBuilder + Harness + HarnessRuntime
- [x] 12 observers, 16 detectors, 14 strategies (all with real logic)
- [x] 31 presets (all wireable via factory)
- [x] Pipeline engine (observe→detect→strategize→act→audit)
- [x] Runtime (channel-based agent↔harness communication)
- [x] Audit trail (immutable append-only, hash chain, signing)
- [x] SQLite, In-memory, and Postgres audit stores
- [x] Human gate (pause/approve/reject/override state machine)
- [x] Checkpoint manager (save/load/evict)
- [x] CLI: `forge init`, `forge run`, `forge doctor`, `forge explain`
- [x] Meta-harness: weakness miner, proposer, validator, A/B testing
- [x] MCP client/server/gateway modules
- [x] Skills registry + composer
- [x] Cloud traits (AWS, Azure, GCP)
- [x] Python bindings — `pip install forge-agent-sdk` (31 presets, Python 3.10–3.14)
- [x] Working Rust example (`examples/basic-agent/`)
- [x] Docker image (multi-stage Dockerfile)
- [x] Dashboard scaffold (Leptos WASM, `forge-dashboard`)
- [x] CI/CD workflow (`.github/workflows/publish-pypi.yml` — builds 15 wheels on tag push)
- [x] Real AgentAdapters for ALL 31 agent types (`packages/forge-adapters/`)
- [x] AdapterFactory — auto-maps AgentType → CliAgent/HttpAgent/PythonAgent/BridgeAgent
- [x] 130 tests, 0 failures

### 🚧 In Progress
- [ ] TypeScript bindings (NAPI-RS)
- [ ] CLI TUI (`forge watch` with ratatui)
- [ ] forge-server route wiring to real SDK

### 📋 Planned
- [ ] VSCode extension
- [ ] Kubernetes operator + Helm chart
- [ ] Plugin marketplace (community registry)
- [ ] SSO/SAML/OIDC (Okta, Azure AD, Google Workspace)
- [ ] EU AI Act compliance report templates
- [ ] LangFuse / W&B Weave native export
- [ ] PagerDuty / OpsGenie integration

---

## License

MIT — see [LICENSE](LICENSE)

---

**Built with Rust 🦀** | 180+ source files | 130 tests | 0 failures
