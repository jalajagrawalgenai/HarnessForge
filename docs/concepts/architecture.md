# Harness Architecture — 4-Layer Pipeline

Forge uses a complete 4-layer architecture that runs on every ingested event:

## Layer 1: Observe (12 dimensions)

Every agent event is flattened via `event_to_observation()` into a JSON object containing all fields that detectors need (tool_name, content, file_path, cost_per_turn, msg_timestamp_ms, etc.). The 12 observers process these:

| Dimension | What It Watches |
|---|---|
| **Token** | Cache hit rate, dedup, compression, waste tokens |
| **Latency** | p50/p95/p99, TTFT, per-tool timing |
| **Cost** | $/operation, budget burn rate, model efficiency |
| **Accuracy** | Test pass rate, lint errors, verification |
| **Security** | Secret leaks, dangerous tools, prompt injection |
| **Reliability** | Error rate, retry frequency, timeout rate |
| **Context Quality** | Info density, redundancy, staleness, context pressure |
| **Orchestration** | Agent tree, routing accuracy |
| **Communication** | Message flow, turn fairness, topic coherence |
| **Memory** | Hit rate, knowledge staleness |
| **Compliance** | PII exposure, audit gaps, gate bypass |
| **Diversity** | Approach similarity, solution coverage |

## Layer 2: Detect (16 detectors)

`detect_from_events()` scans the event history for patterns. Detection runs with:

- **Category-level deduplication**: each category reported once per session (e.g., "accuracy_risk" won't spam)
- **Per-tool loop dedup**: each tool gets loop detection once per session via `loop_detected_tools: HashSet<String>`
- **Direct event scanning**: looks at raw events (tool names, content, timestamps) plus observer output

| Detector | Triggers On | Severity |
|---|---|---|
| LoopDetector | Same tool 4+ calls consecutively, no progress | Warning→Error |
| StaleContextDetector | Context pressure >85% | Warning→Error |
| CostAnomalyDetector | Cost >3× moving average | Warning→Error |
| DeadlockDetector | 2+ agents waiting >60s | Error |
| HallucinationDetector | Reference to non-existent file/API | Warning→Error |
| PromptInjectionDetector | "Ignore previous instructions" patterns | Error |
| SecretLeakDetector | API keys, tokens, private keys (context-aware: `password=`, `password:`) | **Critical** |
| VarietyCollapseDetector | 3+ agents identical outputs >85% | Warning |
| ConversationStallDetector | No events for 45s | Warning→Error |
| GoalDriftDetector | Divergence from original task | Warning→Error |
| ModelMismatchDetector | Complex task → weak model | Warning |
| AccuracyRiskDetector | Code generated, no tests run | Warning |
| RunawayCostDetector | Cost accelerating (2nd derivative) | Warning→Error |
| ResourceExhaustionDetector | Disk/memory > threshold | Warning→Error |
| OutputDegradationDetector | Quality declining 3+ outputs | Warning→Error |
| ComplianceGapDetector | Human gate skip, audit gap | **Critical** |

## Layer 3: Strategize (14 strategies)

When a detection fires, **all 14 strategies** are evaluated against it. The one with the **highest priority** wins — not first-match, best-match. This ensures critical strategies like `circuit_break` (priority 100) aren't blocked by lighter ones like `nudge` (priority 10).

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

## Layer 4: Self-Improve (Meta-Harness)

Runs across sessions to improve the harness itself:

- **POST /v1/meta/improve** — Analyzes completed sessions, finds weakness patterns (high_tool_error_rate, undetected_issues, high_context_pressure, intervention_gap), groups by pattern type
- **GET /v1/meta/weaknesses** — Returns cached weakness patterns from the last improvement cycle
- **GET /v1/meta/edits** — Proposes harness rule changes based on session data (tool_error_threshold, detection_budget)
- **POST /v1/meta/edits/:id/accept** — Accepts a proposed rule change
- **POST /v1/meta/edits/:id/reject** — Rejects a proposed rule change
- **GET /v1/meta/ab-tests** — Returns sessions available for A/B testing

Based on the Self-Harness paper (Shanghai AI Lab, June 2026): 33-60% pass rate improvement.

## Pipeline Flow

```
Agent Event → event_to_observation() → flatten to JSON with all fields
    │
    ├─→ Observers process (12 dimensions)
    │
    ├─→ detect_from_events() scans event history
    │   Uses category dedup + per-tool loop dedup
    │
    ├─→ All 14 strategies evaluated against each detection
    │   Highest priority match is selected
    │
    └─→ Session persisted to ~/.forge/sessions/*.json
        Analysis available via GET /v1/sessions/:id/analysis
```

## Session Analysis

Each session's analysis endpoint returns:

- **Event Log** — every event with seq#, timestamp, type, tool, args, result, duration
- **Hook Trace** — events grouped by hook type (SessionStart, PreToolUse, PostToolUse, etc.)
- **Tool Instances** — every tool call with extracted args and results
- **Prompt Instances** — each prompt with following tool count, latency, tokens
- **Detector Report** — detections grouped by category with severity and confidence
- **Token Analysis** — input/output/cache tokens with clear "estimated" labeling
- **Context Analysis** — pressure history over time
- **Strategy Results** — which strategies were selected and why
