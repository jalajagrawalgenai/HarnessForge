# HarnessForge — Self-Improving Agent Harness SDK

**The world's first harness that watches, detects, intervenes, AND improves itself.**

[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-88%20passed-brightgreen.svg)](.)
[![CI](https://github.com/jalajagrawalgenai/HarnessForge/actions/workflows/ci.yml/badge.svg)](.)

---

## What Is Forge?

Forge is a **self-improving agent harness** that wraps around ANY existing AI agent and provides the missing layer that LangFuse, W&B, Phoenix, and every observability tool cannot: **autonomous intervention and self-improvement.**

```
                    OBSERVABILITY TOOLS (passive)           FORGE (active)
                    ───────────────────────────             ──────────────
LangFuse:           "Cost spiked at 2:34 PM"               Detects spike at turn 3,
                                                            swaps model. Saves $0.12.

W&B Weave:          "Agent made 47 calls"                  Detects loop at turn 4,
                                                            injects nudge. Agent breaks out.

LangSmith:          "Session failed"                        Checkpoints before failure,
                                                            rolls back, retries. Succeeds.

Arize Phoenix:      "Context at 95%"                        Compacts at 75%.
                                                            Crisis prevented.
```

## Why Forge Exists

Every agent framework and every observability tool has the same blind spot:

| Problem | Who Has It |
|---|---|
| Agent loops infinitely, nobody notices | ALL of them |
| Context window fills up, agent degrades silently | ALL of them |
| Wrong model used, costs 10x more | ALL of them |
| API key leaks in output → disaster | ALL of them |
| Agents hallucinate non-existent files | ALL of them |
| Multi-agent deadlock → stuck forever | ALL multi-agent systems |
| No explanation for why decisions happened | ALL of them |

**Forge is the answer to all of these.** It wraps around your existing agents and adds the watching, intervening, and learning layer.

---

## Demo: A Session With Forge

```
┌─ Forge ─── Session: 3b1a9e2c ─── Claude Sonnet ─── Solo ─── 00:04:32 ────────┐
│                                                                                 │
│  ┌─ Conversation ────────────────────────────────────────────────────────────┐ │
│  │                                                                             │ │
│  │  User: Refactor the auth module to use JWT tokens                           │ │
│  │                                                                             │ │
│  │  █ Agent: I'll analyze the current auth module structure first.            │ │
│  │                                                                             │ │
│  │  ▸ Read src/auth/mod.rs (2,340 tokens, 0.8s)                                │ │
│  │  ▸ Read src/auth/session.rs (1,890 tokens, 0.6s)                            │ │
│  │  ▸ Grep "session" in src/ — 12 matches (0.3s)                               │ │
│  │                                                                             │ │
│  │  ⚠ HARNESS [T6]: StaleContext detected. auth/mod.rs re-read.               │ │
│  │     → Strategy: Compact. Context 84% → 58%. Saved 4,200 tokens.            │ │
│  │                                                                             │ │
│  │  █ Agent: Continuing. Let me create the JWT module.                         │ │
│  │  ▸ Write src/auth/jwt.rs (156 lines, 3,100 tokens)                          │ │
│  │  ▸ Write src/auth/jwt_test.rs (89 lines, 1,800 tokens)                      │ │
│  │                                                                             │ │
│  │  🔧 HARNESS [T9]: AccuracyRisk. Tests written but not executed.             │ │
│  │     → Strategy: Nudge. "Run the tests before proceeding."                   │ │
│  │                                                                             │ │
│  │  ▸ Bash cargo test -- auth::jwt (2.3s, 12 passed)                          │ │
│  │  █ Agent: All tests pass. The JWT refactor is complete.                      │ │
│  │                                                                             │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                 │
│  ┌─ Health ────────────────────┐  ┌─ Token Budget ───────────────────────────┐ │
│  │                              │  │                                           │ │
│  │  Token:      🟢 0.92       │  │  Used:    ████████████░░░░░░░░  58%      │ │
│  │  Latency:    🟢 0.88       │  │  Cache:   ██████████░░  23H / 12M        │ │
│  │  Cost:       🟢 0.95       │  │  Cost:    $0.11 / $0.50 budget            │ │
│  │  Accuracy:   🟢 0.91       │  │  Turn:    12                            │ │
│  │  Security:   🟢 1.00       │  │  Session: active ✓                       │ │
│  │  Reliability:🟢 0.97       │  │                                           │ │
│  │  Context:    🟡 0.72       │  │  ┌─ Interventions ──────────────────────┐ │ │
│  │  Memory:     — (n/a)       │  │  │ T6: ⚠ StaleContext → Compact ✓      │ │ │
│  │  Compliance: 🟢 1.00       │  │  │ T9: 🔧 AccuracyRisk → Nudge ✓       │ │ │
│  │  Diversity:  — (n/a)       │  │  └──────────────────────────────────────┘ │ │
│  │  Comm:       — (n/a)       │  │                                           │ │
│  │  Orch:       🟢 0.93       │  │  Checkpoints: ██ 3 saved                  │ │
│  │                              │  │                                           │ │
│  │  ★ Overall:  🟢 0.91       │  │                                           │ │
│  └──────────────────────────────┘  └───────────────────────────────────────────┘ │
│                                                                                 │
│  Ctrl+C quit │ Esc cancel │ Tab mode │ ↑ history │ / search                       │
└─────────────────────────────────────────────────────────────────────────────────┘
```

## Dashboard Screens

### Overview

```
┌─ Forge Dashboard ──────────────────────────────────────────────────────────────┐
│  🔍 Search...                          [⚙ Settings] [🔔 3] [👤 user]          │
├─────────────────────────────────────────────────────────────────────────────────┤
│  📊 Overview  │  📜 Sessions  │  🔍 Audit  │  📈 Analytics  │  🧠 Meta  │  ⚙  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─ Today ───────────────────────────────────────────────────────────────────┐ │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌────────────┐  │ │
│  │  │ Sessions │  │  Tokens  │  │   Cost   │  │  Health  │  │Interventions│ │ │
│  │  │   47     │  │  1.2M    │  │  $4.80   │  │  0.87    │  │ 12 applied  │ │ │
│  │  │  ↑ 12%  │  │  ↓ 8%    │  │  ↓ 15%   │  │  ↑ 0.05  │  │ 10 resolved │ │ │
│  │  └──────────┘  └──────────┘  └──────────┘  └──────────┘  └────────────┘  │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│  ┌─ Active Sessions (3) ──────────────────────────────────────────────────────┐ │
│  │  #142  claude-sonnet  "Refactor auth to JWT"     T12  🟢 0.91  $0.11      │ │
│  │  #143  gpt-4o         "Add payment webhook"      T8   🟡 0.72  $0.08      │ │
│  │  #144  claude-opus    "Security audit report"    T3   🟢 0.95  $0.03      │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│  ┌─ Health Trend ───────────────────────────────┐  ┌─ Recent Detections ──────┐ │
│  │  1.0 ┤                     ●──●              │  │ LoopDetector     12      │ │
│  │  0.8 ┤     ●──●──●──●                        │  │ StaleContext      8      │ │
│  │  0.6 ┤ ●──●                                   │  │ CostAnomaly       3      │ │
│  │      └───┬───┬───┬───┬───┬───┬───            │  │ AccuracyRisk      5      │ │
│  │      Mon Tue Wed Thu Fri Sat Sun              │  │ SecretLeak        0      │ │
│  └───────────────────────────────────────────────┘  └──────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Session Detail

```
┌─ Forge Dashboard ──────────────────────────────────────────────────────────────┐
│  ← Back │ Session #3b1a9e2c │ Claude Sonnet │ Solo │ ✅ Done │ 4.8 min        │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌─ Conversation ─────────────────────┐  ┌─ Health ──────────────────────────┐ │
│  │  [T1] User: "Refactor auth to JWT" │  │  🟢 Token        0.92 ────────    │ │
│  │  [T2] ▸ Read auth/mod.rs          │  │  🟢 Latency      0.88 ────────    │ │
│  │  [T3] ▸ Read auth/session.rs      │  │  🟢 Cost         0.95 ────────    │ │
│  │  [T4] Agent: "I see the Session"  │  │  🟢 Accuracy     0.91 ────────    │ │
│  │  [T5] ▸ Grep "session" in src/    │  │  🟢 Security     1.00 ────────    │ │
│  │  [T6] ⚠ STALE CONTEXT DETECTED   │  │  🟢 Reliability  0.97 ────────    │ │
│  │  [T6] 🔧 Compact applied          │  │  🟡 Context      0.72 ──┬────    │ │
│  │  [T9] 🔧 Nudge: "Run the tests"   │  │  ★ Overall 🟢 0.91              │ │
│  │  [T12] ✅ Session complete        │  └───────────────────────────────────┘ │
│  └───────────────────────────────────┘                                          │
│                                                                                  │
│  ┌─ Intervention Timeline ────────────────────────────────────────────────────┐ │
│  │  T1 ─── T2 ─── T3 ─── T4 ─── T5 ───┬─── T7 ─── T8 ───┬─── T10 ─── T12    │ │
│  │                                     │                   │                   │ │
│  │                                  T6 ⚠🔧               T9 🔧                 │ │
│  │                              StaleContext            AccuracyRisk            │ │
│  │                              → Compact ✓             → Nudge ✓               │ │
│  └──────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                  │
│  ┌─ Token ───────────────────────────┐  ┌─ Audit ─────────────────────────────┐ │
│  │  Input:   22.3K ████████████ 58%  │  │  [T6] DETECT StaleContext (0.91)  │ │
│  │  Output:  11.8K ██████░░░░░ 31%  │  │  [T6] STRATEGY CompactStrategy     │ │
│  │  Cache R:  2.8K ██░░░░░░░░  7%  │  │  [T6] ACTION CompactDone (84→58%)  │ │
│  │  Total:   38.2K                   │  │  [T9] DETECT AccuracyRisk (0.78)  │ │
│  │  Cost: $0.11 / $0.50 budget      │  │  [T9] STRATEGY NudgeStrategy       │ │
│  └───────────────────────────────────┘  └─────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────┘
```

### Meta-Harness Self-Improvement

```
┌─ Forge Dashboard ─── Meta-Harness ─────────────────────────────────────────────┐
│                                                                                 │
│  Agent Type: claude-sonnet  │  Harness: v8  │  247 sessions  │  Improving ✓   │
│                                                                                 │
│  ┌─ Pass Rate Over Time ─────────────────────────────────────────────────────┐ │
│  │  70% │                                                 ●──●──●  v8 (64%)  │ │
│  │  60% │                                    ●──●──●──●  v7 (61%)            │ │
│  │  50% │                         ●──●──●──●  v5 (55%)                       │ │
│  │  40% │ ●──●  v1 (40%) — initial harness                                   │ │
│  │      └───┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────┬────           │ │
│  │        50   75  100  125  150  175  200  225  250                          │ │
│  │                          Sessions                                           │ │
│  └─────────────────────────────────────────────────────────────────────────────┘ │
│                                                                                 │
│  ┌─ Weakness Patterns ──────────────┐  ┌─ Applied Edits ──────────────────────┐ │
│  │  ✅ LateLoopDetection (14)       │  │  v3: Lower loop threshold 6→4       │ │
│  │  ✅ ModelMismatch (9)            │  │      +14% pass rate · p=0.003        │ │
│  │  ✅ ContextOverflow (7)          │  │  v5: Route grep tasks to Haiku       │ │
│  │  ⬜ AccuracyRisk (5)             │  │      +8% pass rate · p=0.012         │ │
│  │     Pending analysis →           │  │  v7: Compact at 80% not 92%          │ │
│  └──────────────────────────────────┘  │      +11% pass rate · p=0.007        │ │
│                                         └──────────────────────────────────────┘ │
│                                                                                 │
│  ┌─ Pending Validation ───────────────────────────────────────────────────────┐ │
│  │  Edit: "Add variety check for multi-agent sessions"                         │ │
│  │  Regression test: ████████████████████░  22/25 pass (baseline: 20/25)     │ │
│  │  Improvement: +8% · p=0.018 · 0 regressions                                │ │
│  │  [✓ Accept]   [✗ Reject]   [✎ Modify]                                      │ │
│  └──────────────────────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────────────────────┘
```

---

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                           │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────────────────┐ │
│  │ forge-cli│   │forge-dash│   │forge-sdk │   │ External Observability│ │
│  │ (ratatui)│   │ (Leptos) │   │ (Rust)   │   │ (LangFuse/Phoenix/etc)│ │
│  └────┬─────┘   └────┬─────┘   └────┬─────┘   └──────────┬───────────┘ │
│       │              │              │                      │             │
│       └──────────────┼──────────────┼──────────────────────┘             │
│                      │              │                                     │
│              ┌───────┴──────────────┴──────────┐                         │
│              │        forge-server (axum)       │                         │
│              │  REST + SSE + WebSocket          │                         │
│              └───────┬──────────────────────────┘                         │
│                      │                                                    │
│       ┌──────────────┼──────────────────────────┐                        │
│       │              │                          │                        │
│  ┌────┴────┐   ┌─────┴──────┐   ┌──────────────┴──────────┐            │
│  │ Session │   │  HARNESS    │   │    META-HARNESS          │            │
│  │ Manager │   │  PIPELINE   │   │                          │            │
│  │         │   │             │   │  Miner→Proposer→         │            │
│  │ Active  │   │ Observe→    │   │  Validator→Registry      │            │
│  │ sessions│   │ Detect→     │   │                          │            │
│  │         │   │ Strategy→   │   │  Runs periodically       │            │
│  │  ┌────┐ │   │ Action→     │   │  Improves Level 1        │            │
│  │  │Agt1│ │   │ Audit       │   │                          │            │
│  │  │Agt2│ │   │             │   └──────────────────────────┘            │
│  │  └────┘ │   │ 12 observers│                                           │
│  └─────────┘   │ 16 detectors│    ┌──────────────────────┐              │
│                │ 14 strategies│    │    AUDIT STORE        │              │
│                └──────┬───────┘    │  SQLite / PostgreSQL  │              │
│                       │            │  + FTS + signing      │              │
│                       │ Event Bus  └──────────────────────┘              │
│                       │ (mpsc)                                           │
│                ┌──────┴───────┐                                          │
│                │  AGENT       │                                          │
│                │  ADAPTER     │                                          │
│                │              │                                          │
│                │ event_tx ───→ harness                                   │
│                │ ←── intervention_rx                                     │
│                └──────┬───────┘                                          │
│                       │                                                  │
│                ┌──────┴───────┐                                          │
│                │  YOUR AGENT  │                                          │
│                │  (any impl)  │                                          │
│                └──────────────┘                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

### Two-Level Self-Improvement

```
┌──────────────────────────────────────────────────────────────────┐
│ LEVEL 2: META-HARNESS — learns across sessions                    │
│                                                                   │
│   Session Audits → WeaknessMiner → HarnessProposer → Validator   │
│                                                                   │
│   "The harness that rewrites the harness"                         │
│   After 50 sessions: pass rate 40% → 64% autonomously             │
└────────────────────────────┬─────────────────────────────────────┘
                             │ improved rules
┌────────────────────────────┴─────────────────────────────────────┐
│ LEVEL 1: SESSION HARNESS — watches within a session               │
│                                                                   │
│   Agent Events → Observe → Detect → Strategize → Act → Audit    │
│                                                                   │
│   "The harness that watches the agent"                            │
│   Real-time detection + intervention + immutable audit            │
└──────────────────────────────────────────────────────────────────┘
```

---

## Packages

| Package | Description | Files |
|---|---|---|
| `forge-sdk` | Public API — types, events, AgentAdapter, HarnessBuilder, presets | 17 |
| `forge-harness` | Pipeline engine — event bus, plugin registry, runtime, checkpoint | 8 |
| `forge-observers` | 12-dimensional real-time watchers + health scorer | 13 |
| `forge-detectors` | 16 real-time issue detectors | 16 |
| `forge-strategies` | 14 autonomous intervention strategies | 14 |
| `forge-audit` | Immutable trail, hash-chain, SQLite/Postgres, replay, export | 10 |
| `forge-meta` | Self-improving meta-harness — mine, propose, validate, A/B test | 6 |
| `forge-bridge` | Model catalog (5 models), cost calculator, LiteLLM client | 4 |
| `forge-mcp` | MCP client, server, gateway, discovery | 4 |
| `forge-skills` | Skill registry, composer, SDK, 3 built-in skills | 3 |
| `forge-cloud` | AWS, Azure, GCP integration stubs | 5 |
| `forge-cli` | Full CLI with 16 commands | 1 |

---

## 12-Dimensional Observation

| Dimension | What It Watches | Health Impact |
|---|---|---|
| **Token** | Cache hit rate, dedup, compression, waste | 🟢 >80% hit rate |
| **Latency** | p50/p99, TTFT, per-tool timing | 🟢 p95 < 5s |
| **Cost** | $/operation, budget burn rate, model efficiency | 🟢 under budget |
| **Accuracy** | Test pass, lint errors, diff quality, verification | 🟢 >90% pass |
| **Orchestration** | Decision trace, agent tree, model routing | 🟢 trace complete |
| **Communication** | Message flow, turn fairness, topic coherence | 🟢 balanced |
| **Security** | Secret leaks, dangerous tools, prompt injection | 🔴 ANY violation |
| **Reliability** | Error rate, retry frequency, timeout rate | 🟢 <5% errors |
| **Context Quality** | Info density, redundancy, staleness | 🟡 monitor trend |
| **Memory** | Hit rate, knowledge staleness, growth | 🟢 >50% hits |
| **Compliance** | PII exposure, data residency, audit gaps | 🔴 ANY violation |
| **Diversity** | Approach similarity, solution coverage | 🟡 >0.3 variety |

---

## 16 Detectors

| Detector | What Triggers It | Severity |
|---|---|---|
| LoopDetector | Same tool × 4 calls, no progress | Warning→Error |
| StaleContextDetector | Same file re-read 3×, pressure >85% | Warning→Error |
| CostAnomalyDetector | Cost >3× moving average | Warning→Error |
| DeadlockDetector | 2+ agents waiting on each other >60s | Error |
| HallucinationDetector | Reference to non-existent file/API | Warning→Error |
| PromptInjectionDetector | "Ignore previous instructions" patterns | Error |
| SecretLeakDetector | API keys, tokens, passwords in output | **Critical** |
| VarietyCollapseDetector | 3+ agents produce identical outputs | Warning |
| ConversationStallDetector | No messages for 45s in multi-agent chat | Warning→Error |
| GoalDriftDetector | Agent diverges from original task | Warning→Error |
| ModelMismatchDetector | Complex task assigned to cheap model | Warning |
| AccuracyRiskDetector | Code generated, no tests executed | Warning |
| RunawayCostDetector | Cost accelerating (2nd derivative) | Warning→Error |
| ResourceExhaustionDetector | Disk/memory > threshold | Warning→Error |
| OutputDegradationDetector | Quality declining over 3+ turns | Warning→Error |
| ComplianceGapDetector | Human gate skipped, audit gap, PII | **Critical** |

---

## 14 Intervention Strategies

| Strategy | What It Does | Priority |
|---|---|---|
| **Nudge** | Inject gentle hint into agent context | 10 (lowest) |
| **Compact** | Trigger 4-layer context compression | 20 |
| **Diversify** | Force agents to use different approaches | 20 |
| **Reroute** | Change agent's next action (graph mode) | 22 |
| **Escalate** | Upgrade model, expand budget | 25 |
| **Rollback** | Restore from checkpoint, retry | 28 |
| **Pause** | Pause agent, notify human | 30 |
| **Interject** | Strong user-like STOP message | 35 |
| **Quarantine** | Route output to sandbox | 40 |
| **Replace** | Kill agent, spawn replacement | 45 |
| **Isolate** | Remove dangerous tools, restrict context | 50 |
| **Degrade** | Switch to cheaper model, remove tools | 15 |
| **Fork** | Split agent into parallel children | 18 |
| **CircuitBreak** | Emergency stop ALL agents | 100 (highest) |

---

## Quick Start

### Installation

```bash
cargo install forge-sdk
```

### Wrap Your Agent (5 minutes)

```rust
use forge_sdk::prelude::*;
use tokio::sync::mpsc;

struct MyAgent;
#[async_trait::async_trait]
impl AgentAdapter for MyAgent {
    fn id(&self) -> String { "my-agent".into() }
    fn agent_type(&self) -> AgentType { AgentType::Solo }
    async fn run(&mut self, task: &str, tx: mpsc::Sender<AgentEvent>, mut rx: mpsc::Receiver<Intervention>) -> Result<AgentOutcome, ForgeError> {
        // 1. Emit events as your agent works
        tx.send(AgentEvent::ThinkingStart { agent_id: self.id(), timestamp: chrono::Utc::now() }).await.ok();
        tx.send(AgentEvent::OutputComplete { agent_id: self.id(), content: format!("Done: {}", task), timestamp: chrono::Utc::now() }).await.ok();

        // 2. Check for harness interventions between turns
        if let Ok(intervention) = rx.try_recv() {
            match intervention {
                Intervention::Nudge { message, .. } => println!("Harness says: {}", message),
                Intervention::CircuitBreak { reason } => return Err(ForgeError::CircuitBroken { reason }),
                _ => {}
            }
        }

        // 3. Emit completion
        tx.send(AgentEvent::Completed { agent_id: self.id(), summary: "Done".into(), timestamp: chrono::Utc::now() }).await.ok();
        Ok(AgentOutcome { success: true, summary: "Task complete".into(), output: None })
    }
}

#[tokio::main]
async fn main() -> Result<(), ForgeError> {
    let harness = Harness::builder()
        .observe(all_observers!())
        .detect(all_detectors!())
        .strategize(all_strategies!())
        .build();

    let mut agent = MyAgent;
    let result = harness.run(&mut agent, "Build JWT auth system").await?;
    println!("Done: interventions={}, detections={}", result.intervention_count, result.detection_count);
    Ok(())
}
```

### CLI

```bash
forge init                          # Scaffold new project
forge run "Implement JWT auth"      # Run agent with harness
forge watch                         # Live TUI health dashboard
forge replay <session-id>           # Replay session step-by-step
forge explain <session-id>          # Human-readable audit report
forge bench --suite standard        # Run benchmark suite
forge improve --agent-type solo     # Run self-improvement cycle
forge serve                         # Start API server + dashboard
forge doctor                        # System dependency check
forge validate                      # Validate harness config
forge diff <s1> <s2>               # Compare two sessions
forge export <id> --format pdf     # Export audit as PDF
```

---

## Self-Improvement

Forge's meta-harness implements the **Self-Harness loop** (validated by Shanghai AI Lab, June 2026 — arXiv:2606.09498):

```
┌───────────────────────────────────────────────┐
│  1. WEAKNESS MINING                           │
│     Analyze session audits → cluster failures │
│                                               │
│  2. HARNESS PROPOSAL                          │
│     Generate minimal, targeted edits          │
│                                               │
│  3. PROPOSAL VALIDATION                       │
│     Regression test → accept only if          │
│     improvement without degradation           │
│                                               │
│  → Loop back to step 1 with improved harness  │
└───────────────────────────────────────────────┘
```

**Results:** 33-60% pass rate improvement across Claude, GPT, Gemini, MiniMax, Qwen, and GLM models without any human intervention.

---

## Tech Stack

| Layer | Technology |
|---|---|
| Core | Rust 1.85+ |
| Async | tokio |
| CLI | clap + ratatui |
| Server | axum + SSE + WebSocket |
| Dashboard | Leptos (Rust WASM) |
| DB (local) | SQLite via sqlx |
| DB (cloud) | PostgreSQL via sqlx |
| Bindings | PyO3 (Python), NAPI-RS (TypeScript) |

---

## Documentation

Full docs: [docs/](docs/) | [mkdocs.yml](mkdocs.yml)

- [Quickstart](docs/getting-started/quickstart.md)
- [Installation](docs/getting-started/installation.md)
- [Architecture](docs/concepts/architecture.md)
- [Self-Improvement](docs/concepts/self-improvement.md)
- [CLI Reference](docs/cli.md)

---

## License

MIT

---

**Built with Rust 🦀** | [GitHub](https://github.com/jalajagrawalgenai/HarnessForge)
