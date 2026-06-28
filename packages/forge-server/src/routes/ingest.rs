//! Event ingestion route — universal entry point for ALL agentic systems.
//!
//! POST /api/v1/ingest/event
//!
//! Accepts events from any agentic framework (Claude Code hooks, LangGraph
//! callbacks, CrewAI middleware, AutoGen observers, etc.). Auto-creates
//! sessions on first contact. Runs the full harness pipeline:
//!   Observe → Detect → Strategize → Intervene
//!
//! POST /api/v1/ingest/transcript
//!
//! Accepts transcript summary data after session completion (token usage,
//! model breakdown, cache metrics) for analytics and audit.

use crate::session::store::{SessionState, SessionStatus};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use forge_sdk::events::AgentEvent;
use serde_json::{json, Value};
use std::sync::Arc;

/// Build a PluginRegistry from a preset string.
/// Maps the preset name to the corresponding Preset enum and builds the registry.
fn build_registry_for_preset(preset_name: &str) -> forge_harness::plugin_registry::PluginRegistry {
    let preset = match preset_name.to_lowercase().as_str() {
        "solo" => forge_sdk::presets::Preset::Solo,
        "claude-code" | "claude" | "claudecode" => forge_sdk::presets::Preset::ClaudeCode,
        "langgraph" => forge_sdk::presets::Preset::LangGraph,
        "crewai" | "crew" => forge_sdk::presets::Preset::CrewAI,
        "autogen" => forge_sdk::presets::Preset::AutoGen,
        "langchain" => forge_sdk::presets::Preset::LangChain,
        "aider" => forge_sdk::presets::Preset::Aider,
        "cline" => forge_sdk::presets::Preset::Cline,
        "continue" => forge_sdk::presets::Preset::Continue,
        "copilot" => forge_sdk::presets::Preset::Copilot,
        "cursor" => forge_sdk::presets::Preset::Cursor,
        "windsurf" => forge_sdk::presets::Preset::Windsurf,
        "devin" => forge_sdk::presets::Preset::Devin,
        _ => forge_sdk::presets::Preset::Solo,
    };
    forge_harness::factory::build_registry_from_preset(&preset)
}

/// POST /api/v1/ingest/event
///
/// Universal event ingestion. Accepts events from any agentic system.
/// Auto-creates sessions when a lifecycle-start event arrives.
/// Runs the harness pipeline (observe → detect → strategize) on every event.
pub async fn ingest_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let hook_name = body
        .get("hookName")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let session_id = body
        .get("sessionId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let agent_id = body
        .get("agentId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let agent_class = body
        .get("agentClass")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let tool_name = body.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
    let timestamp = body.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
    let flags = body.get("flags").cloned().unwrap_or(json!({}));
    let payload = body.get("payload").cloned().unwrap_or(json!({}));

    let starts_session = flags
        .get("startsSession")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let stops_session = flags
        .get("stopsSession")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // ── Auto-create session on first contact ──
    let is_new_session = {
        let sessions = state.store.read().await;
        !sessions.contains_key(session_id)
    };

    if is_new_session {
        let task = payload
            .get("prompt")
            .and_then(|v| v.as_str())
            .unwrap_or("(live agent session)")
            .to_string();

        let agent_type = map_agent_class(agent_class);
        let preset = agent_type.clone();

        let session = SessionState::new(session_id.to_string(), task, agent_type, preset);

        let mut sessions = state.store.write().await;
        sessions.insert(session_id.to_string(), session);
        tracing::info!(
            session_id = %session_id,
            agent_class = %agent_class,
            "Auto-created session from ingest event"
        );
    }

    // ── Convert hook event to AgentEvent ──
    let agent_event = hook_to_agent_event(hook_name, agent_id, tool_name, &payload, timestamp);

    // ── Build registry for this session's preset, filtered by harness config ──
    let registry = {
        let sessions = state.store.read().await;
        let base_registry = sessions
            .get(session_id)
            .map(|s| build_registry_for_preset(&s.preset))
            .unwrap_or_else(|| build_registry_for_preset("solo"));

        let cfg = state.harness_config.read().await;
        // Filter to only enabled observers, detectors, strategies
        let mut filtered = forge_harness::plugin_registry::PluginRegistry::new();
        for obs in base_registry.observers() {
            if cfg.enabled_observers.contains(&obs.name().to_string()) {
                filtered.register_observer(obs.clone());
            }
        }
        for det in base_registry.detectors() {
            if cfg.enabled_detectors.contains(&det.name().to_string()) {
                filtered.register_detector(det.clone());
            }
        }
        for strat in base_registry.strategies() {
            if cfg.enabled_strategies.contains(&strat.name().to_string()) {
                filtered.register_strategy(strat.clone());
            }
        }
        filtered
    };

    // ── Run the harness pipeline ──
    let mut new_observations: Vec<Value> = Vec::new();
    let mut new_detections: Vec<Value> = Vec::new();
    let mut new_strategies: Vec<Value> = Vec::new();
    let mut new_interventions: Vec<Value> = Vec::new();
    let mut health_update: Option<Value> = None;

    if let Some(ref event) = agent_event {
        // ── 1. OBSERVE: Run all observers against this event ──
        for observer in registry.observers() {
            if let Some(obs) = observer.observe(event).await {
                new_observations.push(obs.clone());
            }
        }

        // ── 2. Store event and broadcast ──
        {
            let mut sessions = state.store.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                // Broadcast to WebSocket/SSE consumers
                let _ = s.event_broadcaster.send(event.clone());

                // Store in ring buffer
                s.events.push(event.clone());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                }
                s.event_count += 1;

                // Accumulate observations
                for obs in &new_observations {
                    s.observations.push(obs.clone());
                    // Keep last 200 observation results
                    if s.observations.len() > 200 {
                        s.observations.remove(0);
                    }
                }
            }
        }

        // ── 3. DETECT: Run detectors every 3 events or on key events ──
        let should_detect = {
            let sessions = state.store.read().await;
            sessions
                .get(session_id)
                .map(|s| {
                    s.event_count % 3 == 0
                        || hook_name == "PostToolUse"
                        || hook_name == "PostToolUseFailure"
                })
                .unwrap_or(false)
        };

        if should_detect {
            let observations_snapshot = {
                let sessions = state.store.read().await;
                sessions
                    .get(session_id)
                    .map(|s| s.observations.clone())
                    .unwrap_or_default()
            };

            for detector in registry.detectors() {
                let found = detector.detect(agent_id, &observations_snapshot).await;
                for issue in &found {
                    new_detections.push(serde_json::json!({
                        "category": issue.category_name(),
                        "severity": format!("{:?}", issue.severity),
                        "confidence": issue.confidence,
                        "description": issue.description,
                    }));

                    // ── 4. STRATEGIZE: Run strategies for each detection ──
                    for strategy in registry.strategies() {
                        if let Some(result) = strategy.evaluate(issue).await {
                            new_strategies.push(serde_json::json!({
                                "strategy": strategy.name(),
                                "detection": issue.category_name(),
                                "intervention": format!("{:?}", result.intervention),
                            }));
                            new_interventions.push(serde_json::json!({
                                "strategy": strategy.name(),
                                "action": format!("{:?}", result.intervention),
                            }));
                            break; // First matching strategy wins
                        }
                    }
                }
            }

            // Store detections, strategies, and interventions (as JSON for API)
            {
                let mut sessions = state.store.write().await;
                if let Some(s) = sessions.get_mut(session_id) {
                    for d in &new_detections {
                        s.detections.push(d.clone());
                    }
                    for strat in &new_strategies {
                        s.strategy_results.push(strat.clone());
                    }
                    for iv_json in &new_interventions {
                        s.interventions.push(iv_json.clone());
                    }

                    // Trim to reasonable sizes
                    if s.detections.len() > 100 {
                        s.detections.remove(0);
                    }
                    if s.strategy_results.len() > 100 {
                        s.strategy_results.remove(0);
                    }
                    if s.interventions.len() > 100 {
                        s.interventions.remove(0);
                    }
                }
            }
        }

        // ── 5. HEALTH: Compute health score from observations ──
        {
            let sessions = state.store.read().await;
            if let Some(s) = sessions.get(session_id) {
                let dims = compute_health_dimensions(&s.observations);
                let overall = compute_overall_health(&dims);
                let trend = "stable"; // simplified — full trend tracking needs previous score

                health_update = Some(serde_json::json!({
                    "overall": overall,
                    "trend": trend,
                    "dimensions": {
                        "token_efficiency": dims.token_efficiency,
                        "latency": dims.latency,
                        "cost": dims.cost,
                        "accuracy": dims.accuracy,
                        "orchestration": dims.orchestration,
                        "security": dims.security,
                        "reliability": dims.reliability,
                        "context_quality": dims.context_quality,
                        "compliance": dims.compliance,
                    },
                }));
            }
        }

        // Store health score
        if let Some(ref hu) = health_update {
            let mut sessions = state.store.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.health_score = Some(forge_sdk::types::health::HealthScore {
                    agent_id: agent_id.to_string(),
                    overall: hu["overall"].as_f64().unwrap_or(1.0),
                    dimensions: forge_sdk::types::health::HealthDimensions {
                        token_efficiency: hu["dimensions"]["token_efficiency"]
                            .as_f64()
                            .unwrap_or(1.0),
                        latency: hu["dimensions"]["latency"].as_f64().unwrap_or(1.0),
                        cost: hu["dimensions"]["cost"].as_f64().unwrap_or(1.0),
                        accuracy: hu["dimensions"]["accuracy"].as_f64().unwrap_or(1.0),
                        orchestration: hu["dimensions"]["orchestration"].as_f64().unwrap_or(1.0),
                        security: hu["dimensions"]["security"].as_f64().unwrap_or(1.0),
                        reliability: hu["dimensions"]["reliability"].as_f64().unwrap_or(1.0),
                        context_quality: hu["dimensions"]["context_quality"]
                            .as_f64()
                            .unwrap_or(1.0),
                        compliance: hu["dimensions"]["compliance"].as_f64().unwrap_or(1.0),
                        communication: None,
                        memory: None,
                        diversity: None,
                    },
                    trend: forge_sdk::types::health::HealthTrend::Stable,
                });
            }
        }
    }

    // ── Update session status and cumulative analysis ──
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            if starts_session && s.status == SessionStatus::Pending {
                s.status = SessionStatus::Running;
            }
            if stops_session {
                s.status = SessionStatus::Completed;
                s.completed_at = Some(Utc::now());
                // Record stop reason from payload
                s.stop_reason = payload
                    .get("stop_reason")
                    .or(payload.get("reason"))
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .or_else(|| Some(format!("Session ended via {}", hook_name)));
            }
            if hook_name == "StopFailure" {
                s.status = SessionStatus::Failed;
                s.completed_at = Some(Utc::now());
                s.stop_reason = payload
                    .get("error")
                    .and_then(|v| v.as_str())
                    .map(|e| format!("Failed: {}", e));
            }
            if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                if s.task == "(live agent session)" && !prompt.is_empty() {
                    s.task = prompt.to_string();
                }
            }

            // ── Cumulative analysis tracking ──
            match hook_name {
                "UserPromptSubmit" => {
                    s.user_prompt_count += 1;
                }
                "PostToolUse" => {
                    let tool = if tool_name.is_empty() {
                        payload
                            .get("tool_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                    } else {
                        tool_name
                    };
                    *s.tool_counts.entry(tool.to_string()).or_insert(0) += 1;

                    // Track errors
                    if payload
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        *s.tool_errors.entry(tool.to_string()).or_insert(0) += 1;
                    }

                    // Track tool duration from payload
                    if let Some(ms) = payload.get("duration_ms").and_then(|v| v.as_u64()) {
                        s.total_tool_ms += ms;
                    }

                    // Track tokens from tool response
                    if let Some(usage) = payload.get("usage") {
                        if let Some(input) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                            s.total_input_tokens += input;
                        }
                        if let Some(output) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                            s.total_output_tokens += output;
                        }
                    }
                }
                "PostToolUseFailure" => {
                    let tool = if tool_name.is_empty() {
                        payload
                            .get("tool_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                    } else {
                        tool_name
                    };
                    *s.tool_errors.entry(tool.to_string()).or_insert(0) += 1;
                    *s.tool_counts.entry(tool.to_string()).or_insert(0) += 1;
                }
                "PreCompact" => {
                    if let Some(ratio) = payload.get("context_ratio").and_then(|v| v.as_f64()) {
                        s.context_pressure_history.push(ratio);
                    }
                }
                "SubagentStart" => {
                    s.subagent_count += 1;
                }
                _ => {}
            }

            // Track tokens from transcript-style events
            if let Some(AgentEvent::TokenUsage {
                input,
                output,
                cache_read,
                cache_write,
                model,
                ..
            }) = &agent_event
            {
                s.total_input_tokens += input;
                s.total_output_tokens += output;
                s.total_cache_read += cache_read;
                s.total_cache_write += cache_write;
                if s.model_name.is_none() && !model.is_empty() && model != "unknown" {
                    s.model_name = Some(model.clone());
                }
            }
        }
    }

    // ── Build response ──
    Json(json!({
        "status": "ok",
        "sessionId": session_id,
        "hookName": hook_name,
        "ingested": true,
        "pipeline": {
            "observations": new_observations,
            "detections": new_detections,
            "strategies": new_strategies,
            "interventions": new_interventions,
            "health": health_update,
        }
    }))
}

/// POST /api/v1/ingest/transcript
///
/// Accepts transcript summary after session completion.
/// Stores token usage, model breakdown, cache metrics.
pub async fn ingest_transcript(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let session_id = body.get("sessionId").and_then(|v| v.as_str()).unwrap_or("");

    tracing::info!(
        session_id = %session_id,
        "Received transcript data"
    );

    // Store transcript metrics in session
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            let input = body
                .get("totalInputTokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = body
                .get("totalOutputTokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = body
                .get("totalCacheRead")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_write = body
                .get("totalCacheWrite")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let models: Vec<String> = body
                .get("models")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            if input > 0 || output > 0 {
                let model = models.first().cloned().unwrap_or_else(|| "unknown".into());
                // Set model name from transcript
                if s.model_name.is_none() && model != "unknown" {
                    s.model_name = Some(model.clone());
                }
                // Accumulate token totals
                s.total_input_tokens += input;
                s.total_output_tokens += output;
                s.total_cache_read += cache_read;
                s.total_cache_write += cache_write;

                let token_event = AgentEvent::TokenUsage {
                    agent_id: s.id.clone(),
                    input,
                    output,
                    cache_read,
                    cache_write,
                    model,
                    timestamp: Utc::now(),
                };
                s.events.push(token_event.clone());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                }
                let _ = s.event_broadcaster.send(token_event);
            }
        }
    }

    Json(json!({
        "status": "ok",
        "sessionId": session_id,
        "synced": true
    }))
}

/// POST /api/v1/ingest/batch
///
/// Batch ingest for Cursor, Antigravity, and other agents without native hooks.
/// Body: { "agentClass": "cursor", "events": [ { "hookName": "SessionStart", ... } ] }
pub async fn ingest_batch(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let agent_class = body
        .get("agentClass")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let events = body
        .get("events")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let mut ingested = 0u64;

    for ev in &events {
        let hook_name = ev
            .get("hookName")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let session_id = ev
            .get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let agent_id = ev
            .get("agentId")
            .and_then(|v| v.as_str())
            .unwrap_or("root");
        let tool_name = ev.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
        let payload = ev.get("payload").cloned().unwrap_or(json!({}));
        let flags = ev.get("flags").cloned().unwrap_or(json!({}));
        let starts_session = flags
            .get("startsSession")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let stops_session = flags
            .get("stopsSession")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Auto-create session
        let is_new = {
            let sessions = state.store.read().await;
            !sessions.contains_key(session_id)
        };
        if is_new {
            let task = payload
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("(batch)")
                .to_string();
            let agent_type = map_agent_class(agent_class);
            let session = SessionState::new(
                session_id.to_string(),
                task,
                agent_type.clone(),
                agent_type,
            );
            let mut sessions = state.store.write().await;
            sessions.insert(session_id.to_string(), session);
            tracing::info!(session_id = %session_id, agent_class = %agent_class, "Batch-created session");
        }

        let agent_event =
            hook_to_agent_event(hook_name, agent_id, tool_name, &payload, 0);
        if let Some(event) = agent_event {
            let mut sessions = state.store.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.events.push(event.clone());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                }
                s.event_count += 1;
                let _ = s.event_broadcaster.send(event);
                if starts_session && s.status == SessionStatus::Pending {
                    s.status = SessionStatus::Running;
                }
                if stops_session {
                    s.status = SessionStatus::Completed;
                    s.completed_at = Some(Utc::now());
                }
            }
            ingested += 1;
        }
    }

    Json(json!({
        "status": "ok",
        "agentClass": agent_class,
        "eventsReceived": events.len(),
        "eventsIngested": ingested,
    }))
}

/// GET /api/v1/ingest/status
///
/// Returns ingestion stats: active sessions, event counts, pipeline metrics.
pub async fn ingest_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let total = sessions.len();
    let running = sessions
        .values()
        .filter(|s| s.status == SessionStatus::Running)
        .count();
    let total_events: usize = sessions.values().map(|s| s.events.len()).sum();
    let total_observations: usize = sessions.values().map(|s| s.observations.len()).sum();
    let total_detections: usize = sessions.values().map(|s| s.detections.len()).sum();
    let total_interventions: usize = sessions.values().map(|s| s.interventions.len()).sum();
    let auto_created = sessions
        .values()
        .filter(|s| s.task == "(live agent session)")
        .count();

    Json(json!({
        "status": "ok",
        "activeSessions": running,
        "totalSessions": total,
        "totalEventsInRing": total_events,
        "totalObservations": total_observations,
        "totalDetections": total_detections,
        "totalInterventions": total_interventions,
        "autoCreatedSessions": auto_created,
        "message": if running > 0 {
            format!("{} agent(s) being monitored", running)
        } else {
            "Waiting for agent activity...".into()
        }
    }))
}

// ── Helpers ──

/// Map an agent class string (from hook envelope) to our AgentType string.
fn map_agent_class(ac: &str) -> String {
    match ac.to_lowercase().as_str() {
        "claude-code" | "claudecode" | "claude" => "claude-code",
        "langgraph" => "langgraph",
        "crewai" | "crew" => "crewai",
        "autogen" => "autogen",
        "langchain" => "langchain",
        "openai-swarm" | "swarm" => "openai-swarm",
        "semantic-kernel" | "sk" => "semantic-kernel",
        "haystack" => "haystack",
        "dspy" => "dspy",
        "llamaindex" | "llama-index" => "llamaindex",
        "taskweaver" => "taskweaver",
        "agno" => "agno",
        "atomic-agents" | "atomic" => "atomic-agents",
        "bee-agent" | "bee" => "bee-agent",
        "pydantic-ai" | "pydantic" => "pydantic-ai",
        "aider" => "aider",
        "cline" => "cline",
        "continue" => "continue",
        "vercel-ai" | "vercel" => "vercel-ai",
        "copilot" => "copilot",
        "cursor" => "cursor",
        "windsurf" => "windsurf",
        "devin" => "devin",
        "amazon-q" | "q" => "amazon-q",
        "replit-agent" | "replit" => "replit-agent",
        "pearai" | "pear" => "pearai",
        "bolt-new" | "bolt" => "bolt-new",
        "lovable" => "lovable",
        "v0" => "v0",
        "antigravity" | "anti-gravity" | "anti_gravity" => "antigravity",
        "solo" | "custom" | "default" => "solo",
        _ => "solo",
    }
    .to_string()
}

/// Compute a weighted overall health score from dimensions.
type HealthDimFn = dyn Fn(&forge_sdk::types::health::HealthDimensions) -> f64;

fn compute_overall_health(dims: &forge_sdk::types::health::HealthDimensions) -> f64 {
    let weights: &[(&HealthDimFn, f64)] = &[
        (&|d| d.token_efficiency, 0.12),
        (&|d| d.latency, 0.08),
        (&|d| d.cost, 0.10),
        (&|d| d.accuracy, 0.12),
        (&|d| d.orchestration, 0.08),
        (&|d| d.security, 0.12),
        (&|d| d.reliability, 0.08),
        (&|d| d.context_quality, 0.08),
        (&|d| d.compliance, 0.08),
    ];
    let mut total = 0.0;
    let mut weight_sum = 0.0;
    for (getter, weight) in weights {
        total += getter(dims) * weight;
        weight_sum += weight;
    }
    if weight_sum > 0.0 {
        total / weight_sum
    } else {
        1.0
    }
}

/// Compute health dimensions from accumulated observations.
fn compute_health_dimensions(observations: &[Value]) -> forge_sdk::types::health::HealthDimensions {
    let mut dims = forge_sdk::types::health::HealthDimensions {
        token_efficiency: 1.0,
        latency: 1.0,
        cost: 1.0,
        accuracy: 1.0,
        orchestration: 1.0,
        security: 1.0,
        reliability: 1.0,
        context_quality: 1.0,
        compliance: 1.0,
        communication: None,
        memory: None,
        diversity: None,
    };

    for obs in observations {
        if let Some(dim) = obs.get("dimension").and_then(|v| v.as_str()) {
            match dim {
                "token" => {
                    if let Some(rate) = obs.get("cache_hit_rate").and_then(|v| v.as_f64()) {
                        dims.token_efficiency = rate;
                    }
                }
                "latency" => {
                    if let Some(ms) = obs.get("avg_ms").and_then(|v| v.as_f64()) {
                        // Normalize: <500ms = 1.0, >5000ms = 0.0
                        dims.latency = (1.0 - (ms / 5000.0).min(1.0)).max(0.0);
                    }
                }
                "cost" => {
                    if let Some(rate) = obs.get("cost_per_turn").and_then(|v| v.as_f64()) {
                        // Normalize: $0 cost = 1.0, high cost = lower
                        dims.cost = (1.0 - (rate / 2.0).min(1.0)).max(0.0);
                    }
                }
                "security" => {
                    if let Some(issues) = obs.get("issues_found").and_then(|v| v.as_u64()) {
                        dims.security = if issues == 0 { 1.0 } else { 0.7 };
                    }
                }
                "orchestration" => {
                    if let Some(score) = obs.get("multi_agent_score").and_then(|v| v.as_f64()) {
                        dims.orchestration = score;
                    }
                }
                "context_quality" => {
                    if let Some(ratio) = obs.get("context_ratio").and_then(|v| v.as_f64()) {
                        dims.context_quality = (1.0 - ratio.min(1.0)).max(0.0);
                    }
                }
                _ => {}
            }
        }
    }

    dims
}

/// Convert a Claude Code / agentic hook event into a harness AgentEvent.
fn hook_to_agent_event(
    hook_name: &str,
    agent_id: &str,
    tool_name: &str,
    payload: &Value,
    timestamp_ms: u64,
) -> Option<AgentEvent> {
    let ts = if timestamp_ms > 0 {
        let secs = (timestamp_ms / 1000) as i64;
        let nsecs = ((timestamp_ms % 1000) * 1_000_000) as u32;
        chrono::DateTime::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };

    match hook_name {
        "SessionStart" | "Setup" => {
            let task = payload
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("(session started)");
            Some(AgentEvent::Started {
                agent_id: agent_id.to_string(),
                task: task.to_string(),
                timestamp: ts,
            })
        }
        "SessionEnd" | "Stop" => Some(AgentEvent::Completed {
            agent_id: agent_id.to_string(),
            summary: "Session completed".into(),
            timestamp: ts,
        }),
        "StopFailure" => {
            let error = payload
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            Some(AgentEvent::Failed {
                agent_id: agent_id.to_string(),
                error: error.to_string(),
                timestamp: ts,
            })
        }
        "UserPromptSubmit" => {
            let text = payload
                .get("prompt")
                .or(payload.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Some(AgentEvent::MessageSent {
                from: "user".into(),
                to: vec![agent_id.to_string()],
                content: forge_sdk::events::MessageContent::Text(text.to_string()),
                timestamp: ts,
            })
        }
        "PreToolUse" => {
            let args = payload.get("tool_input").cloned().unwrap_or(json!({}));
            Some(AgentEvent::ToolCallStart {
                agent_id: agent_id.to_string(),
                tool: tool_name.to_string(),
                args,
                timestamp: ts,
            })
        }
        "PostToolUse" => {
            // Extract tool result from Claude Code's hook payload
            let tool_response = payload.get("tool_response");
            let content = tool_response.and_then(|v| v.as_str()).unwrap_or("");
            // If content is empty, try to describe what was done
            let content = if content.is_empty() {
                let ti = payload.get("tool_input").cloned().unwrap_or(json!({}));
                match tool_name {
                    "Write" | "Edit" => {
                        let path = ti.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
                        format!("Modified {}", path)
                    }
                    "Read" => {
                        let path = ti.get("file_path").and_then(|v| v.as_str()).unwrap_or("");
                        format!("Read {}", path)
                    }
                    "Bash" | "PowerShell" => {
                        let cmd = ti.get("command").and_then(|v| v.as_str()).unwrap_or("");
                        format!("Ran: {}", if cmd.len() > 60 { &cmd[..60] } else { cmd })
                    }
                    "Grep" => {
                        let pattern = ti.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                        format!("Searched: {}", pattern)
                    }
                    _ => format!("{} completed", tool_name),
                }
            } else {
                content.to_string()
            };

            let is_error = payload
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Extract token count from usage data in payload
            let token_count = payload
                .get("usage")
                .and_then(|v| v.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .or_else(|| {
                    payload
                        .get("usage")
                        .and_then(|v| v.get("output_tokens"))
                        .and_then(|v| v.as_u64())
                })
                .unwrap_or(0);

            Some(AgentEvent::ToolCallEnd {
                agent_id: agent_id.to_string(),
                tool: tool_name.to_string(),
                result: forge_sdk::events::ToolResult {
                    content,
                    is_error,
                    duration_ms: 0,
                    token_count,
                },
                timestamp: ts,
            })
        }
        "PostToolUseFailure" => Some(AgentEvent::ToolCallEnd {
            agent_id: agent_id.to_string(),
            tool: tool_name.to_string(),
            result: forge_sdk::events::ToolResult {
                content: payload
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tool failed")
                    .to_string(),
                is_error: true,
                duration_ms: 0,
                token_count: 0,
            },
            timestamp: ts,
        }),
        "PreCompact" => {
            let ratio = payload
                .get("context_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.85);
            Some(AgentEvent::ContextPressure {
                agent_id: agent_id.to_string(),
                current_ratio: ratio,
                trend: 0.05,
                timestamp: ts,
            })
        }
        "SubagentStart" => {
            let child_id = payload
                .get("subagent_id")
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id);
            let task = payload
                .get("subagent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("subagent task");
            Some(AgentEvent::Forked {
                parent_id: agent_id.to_string(),
                child_id: child_id.to_string(),
                task: task.to_string(),
                timestamp: ts,
            })
        }
        "SubagentStop" => Some(AgentEvent::Completed {
            agent_id: payload
                .get("subagent_id")
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id)
                .to_string(),
            summary: "Subagent finished".into(),
            timestamp: ts,
        }),
        "Notification" => {
            let text = payload
                .get("message")
                .or(payload.get("notification"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !text.is_empty() {
                Some(AgentEvent::OutputDelta {
                    agent_id: agent_id.to_string(),
                    text: text.to_string(),
                    timestamp: ts,
                })
            } else {
                None
            }
        }
        // Other events — store for audit
        _ => Some(AgentEvent::OutputDelta {
            agent_id: agent_id.to_string(),
            text: format!(
                "[{}] {}",
                hook_name,
                payload
                    .get("summary")
                    .or(payload.get("description"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
            ),
            timestamp: ts,
        }),
    }
}
