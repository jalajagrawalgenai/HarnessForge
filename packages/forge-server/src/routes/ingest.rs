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
use std::collections::HashMap;
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

/// Run a single pipeline cycle: observe → detect → strategize → health.
/// Shared by ingest_event and ingest_batch.
async fn run_pipeline_cycle(
    state: &Arc<AppState>,
    session_id: &str,
    agent_id: &str,
    hook_name: &str,
    registry: &forge_harness::plugin_registry::PluginRegistry,
    event: &AgentEvent,
    payload: &Value,
    starts_session: bool,
    stops_session: bool,
) -> Value {
    let mut new_observations: Vec<Value> = Vec::new();
    let mut new_detections: Vec<Value> = Vec::new();
    let mut new_strategies: Vec<Value> = Vec::new();
    let mut new_interventions: Vec<Value> = Vec::new();
    let mut health_update: Option<Value> = None;

    // 1. OBSERVE
    for observer in registry.observers() {
        if let Some(obs) = observer.observe(event).await {
            new_observations.push(obs.clone());
        }
    }

    // 2. STORE + BROADCAST
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            let _ = s.event_broadcaster.send(event.clone());
            s.events.push(event.clone());
            s.event_hooks.push(hook_name.to_string());
            if s.events.len() > 1000 {
                s.events.remove(0);
                s.event_hooks.remove(0);
            }
            s.event_count += 1;
            for obs in &new_observations {
                s.observations.push(obs.clone());
                if s.observations.len() > 500 {
                    s.observations.remove(0);
                }
            }
            // ALSO inject flattened event data into observations so detectors can access
            // ALL event fields (tool_name, content, file_path, etc.) directly.
            // Observers produce computed metrics — this provides the raw event context.
            if let Some(flat) = event_to_observation(event) {
                s.observations.push(flat);
                if s.observations.len() > 500 {
                    s.observations.remove(0);
                }
            }
        }
    }

    // 3. DETECT + STRATEGIZE
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
                new_detections.push(json!({
                    "category": issue.category_name(),
                    "severity": format!("{:?}", issue.severity),
                    "confidence": issue.confidence,
                    "description": issue.description,
                }));
                // Try ALL strategies, collect results, pick the BEST (highest priority)
                let mut best_strategy: Option<(u32, Value, Value)> = None;
                for strategy in registry.strategies() {
                    if let Some(result) = strategy.evaluate(issue).await {
                        let priority = result.priority;
                        let s_json = json!({
                            "strategy": strategy.name(),
                            "detection": issue.category_name(),
                            "intervention": format!("{:?}", result.intervention),
                            "priority": priority,
                            "reasoning": result.reasoning,
                        });
                        let i_json = json!({
                            "strategy": strategy.name(),
                            "action": format!("{:?}", result.intervention),
                            "priority": priority,
                        });
                        match best_strategy {
                            None => best_strategy = Some((priority, s_json, i_json)),
                            Some((p, _, _)) if priority > p => best_strategy = Some((priority, s_json, i_json)),
                            _ => {}
                        }
                    }
                }
                if let Some((_, s_json, i_json)) = best_strategy {
                    new_strategies.push(s_json);
                    new_interventions.push(i_json);
                }
            }
        }
        // Direct event-based detection
        {
            let sessions = state.store.read().await;
            if let Some(s) = sessions.get(session_id) {
                let (dd, ds, di) = detect_from_events(s, registry);
                new_detections.extend(dd);
                new_strategies.extend(ds);
                new_interventions.extend(di);
            }
        }
        // Store
        {
            let mut sessions = state.store.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                for d in &new_detections {
                    s.detections.push(d.clone());
                }
                for st in &new_strategies {
                    s.strategy_results.push(st.clone());
                }
                for iv in &new_interventions {
                    s.interventions.push(iv.clone());
                }
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

    // 4. HEALTH
    {
        let sessions = state.store.read().await;
        if let Some(s) = sessions.get(session_id) {
            let dims = compute_health_dimensions(&s.observations);
            let overall = compute_overall_health(&dims);
            health_update = Some(json!({
                "overall": overall,
                "trend": "stable",
                "dimensions": {
                    "token_efficiency": dims.token_efficiency,
                    "latency": dims.latency, "cost": dims.cost,
                    "accuracy": dims.accuracy, "orchestration": dims.orchestration,
                    "security": dims.security, "reliability": dims.reliability,
                    "context_quality": dims.context_quality, "compliance": dims.compliance,
                },
            }));
        }
    }
    if let Some(ref hu) = health_update {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            s.health_score = Some(forge_sdk::types::health::HealthScore {
                agent_id: agent_id.to_string(),
                overall: hu["overall"].as_f64().unwrap_or(1.0),
                dimensions: forge_sdk::types::health::HealthDimensions {
                    token_efficiency: hu["dimensions"]["token_efficiency"].as_f64().unwrap_or(1.0),
                    latency: hu["dimensions"]["latency"].as_f64().unwrap_or(1.0),
                    cost: hu["dimensions"]["cost"].as_f64().unwrap_or(1.0),
                    accuracy: hu["dimensions"]["accuracy"].as_f64().unwrap_or(1.0),
                    orchestration: hu["dimensions"]["orchestration"].as_f64().unwrap_or(1.0),
                    security: hu["dimensions"]["security"].as_f64().unwrap_or(1.0),
                    reliability: hu["dimensions"]["reliability"].as_f64().unwrap_or(1.0),
                    context_quality: hu["dimensions"]["context_quality"].as_f64().unwrap_or(1.0),
                    compliance: hu["dimensions"]["compliance"].as_f64().unwrap_or(1.0),
                    communication: None,
                    memory: None,
                    diversity: None,
                },
                trend: forge_sdk::types::health::HealthTrend::Stable,
            });
        }
    }

    // 5. SESSION STATUS + CUMULATIVE TRACKING
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            if starts_session && s.status == SessionStatus::Pending {
                s.status = SessionStatus::Running;
            }
            if stops_session {
                s.status = SessionStatus::Completed;
                s.completed_at = Some(Utc::now());
            }
            if hook_name == "StopFailure" {
                s.status = SessionStatus::Failed;
                s.completed_at = Some(Utc::now());
            }
            if let Some(p) = payload.get("prompt").and_then(|v| v.as_str()) {
                if s.task == "(live agent session)" && !p.is_empty() {
                    s.task = p.to_string();
                }
            }
            // Cumulative tracking
            match hook_name {
                "UserPromptSubmit" => {
                    s.user_prompt_count += 1;
                    // Capture the user's prompt text for audit trail
                    if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                        if !prompt.is_empty() {
                            s.prompt_history.push(prompt.to_string());
                            if s.prompt_history.len() > 50 {
                                s.prompt_history.remove(0);
                            }
                        }
                    }
                }
                "SessionStart" => {
                    // Extract model from SessionStart payload
                    if s.model_name.is_none() {
                        if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
                            if !model.is_empty() && model != "unknown" {
                                s.model_name = Some(model.to_string());
                            }
                        }
                    }
                    // Set task from initial prompt (but do NOT push to prompt_history —
                    // UserPromptSubmit handles that to avoid duplicate entries)
                    if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                        if !prompt.is_empty() && s.task == "(live agent session)" {
                            s.task = prompt.to_string();
                        }
                    }
                }
                "PostToolUse" => {
                    let tool = payload
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    *s.tool_counts.entry(tool.to_string()).or_insert(0) += 1;
                    if payload
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        *s.tool_errors.entry(tool.to_string()).or_insert(0) += 1;
                    }
                    if let Some(ms) = payload.get("duration_ms").and_then(|v| v.as_u64()) {
                        s.total_tool_ms += ms;
                    }
                    if let Some(u) = payload.get("usage") {
                        if let Some(i) = u.get("input_tokens").and_then(|v| v.as_u64()) {
                            s.total_input_tokens += i;
                        }
                        if let Some(o) = u.get("output_tokens").and_then(|v| v.as_u64()) {
                            s.total_output_tokens += o;
                        }
                    }
                }
                "PostToolUseFailure" => {
                    let tool = payload
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown");
                    *s.tool_errors.entry(tool.to_string()).or_insert(0) += 1;
                    *s.tool_counts.entry(tool.to_string()).or_insert(0) += 1;
                }
                "PreCompact" => {
                    if let Some(r) = payload.get("context_ratio").and_then(|v| v.as_f64()) {
                        s.context_pressure_history.push(r);
                    }
                }
                "SubagentStart" => {
                    s.subagent_count += 1;
                }
                _ => {}
            }
            if let AgentEvent::TokenUsage {
                input,
                output,
                cache_read,
                cache_write,
                model,
                ..
            } = event
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

    json!({
        "observations": new_observations,
        "detections": new_detections,
        "strategies": new_strategies,
        "interventions": new_interventions,
        "health": health_update,
    })
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
            // Run both formal detectors AND direct event-based detection
            let observations_snapshot = {
                let sessions = state.store.read().await;
                sessions
                    .get(session_id)
                    .map(|s| s.observations.clone())
                    .unwrap_or_default()
            };

            // 1. Formal detectors (use observer output)
            for detector in registry.detectors() {
                let found = detector.detect(agent_id, &observations_snapshot).await;
                for issue in &found {
                    new_detections.push(serde_json::json!({
                        "category": issue.category_name(),
                        "severity": format!("{:?}", issue.severity),
                        "confidence": issue.confidence,
                        "description": issue.description,
                    }));
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
                            break;
                        }
                    }
                }
            }

            // 2. Direct event-based detection (sync — no strategy evaluation)
            let (direct_detections, direct_strategies_from_ev, direct_interventions_from_ev) = {
                let sessions = state.store.read().await;
                if let Some(s) = sessions.get(session_id) {
                    detect_from_events(s, &registry)
                } else {
                    (vec![], vec![], vec![])
                }
            };
            new_detections.extend(direct_detections);
            new_strategies.extend(direct_strategies_from_ev);
            new_interventions.extend(direct_interventions_from_ev);
            // Store detections, strategies, and interventions (as JSON for API)
            {
                let mut sessions = state.store.write().await;
                if let Some(s) = sessions.get_mut(session_id) {
                    for d in &new_detections {
                        s.detections.push(d.clone());
                        // Track categories to prevent duplicates
                        if let Some(cat) = d.get("category").and_then(|c| c.as_str()) {
                            s.reported_categories.insert(cat.to_string());
                        }
                        // Track loop detections per tool to prevent duplicates
                        if d.get("category").and_then(|c| c.as_str()) == Some("loop") {
                            if let Some(desc) = d.get("description").and_then(|c| c.as_str()) {
                                // Extract tool name from description: "Loop detected: 'Bash' called..."
                                if let Some(start) = desc.find('\'') {
                                    if let Some(end) = desc[start+1..].find('\'') {
                                        s.loop_detected_tools.insert(desc[start+1..start+1+end].to_string());
                                    }
                                }
                            }
                        }
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
                // Auto-detect model from environment if still unknown
                if s.model_name.is_none() {
                    s.model_name = detect_model_from_env();
                }
                // Estimate tokens if no transcript data was received
                if s.total_input_tokens == 0 && s.total_output_tokens == 0 {
                    let (est_in, est_out) = estimate_tokens(s);
                    s.total_input_tokens = est_in;
                    s.total_output_tokens = est_out;
                    s.stop_reason = Some(format!(
                        "{} (tokens estimated from {} events, {} tool calls)",
                        s.stop_reason.as_deref().unwrap_or("Session ended"),
                        s.event_count,
                        s.tool_counts.values().sum::<u64>()
                    ));
                }
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
                    // Capture the user's prompt text for audit trail
                    if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                        if !prompt.is_empty() {
                            s.prompt_history.push(prompt.to_string());
                            if s.prompt_history.len() > 50 {
                                s.prompt_history.remove(0);
                            }
                        }
                    }
                }
                "SessionStart" => {
                    // Extract model from SessionStart payload
                    if s.model_name.is_none() {
                        if let Some(model) = payload.get("model").and_then(|v| v.as_str()) {
                            if !model.is_empty() && model != "unknown" {
                                s.model_name = Some(model.to_string());
                            }
                        }
                    }
                    // Set task from initial prompt (but do NOT push to prompt_history —
                    // UserPromptSubmit handles that to avoid duplicate entries)
                    if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                        if !prompt.is_empty() && s.task == "(live agent session)" {
                            s.task = prompt.to_string();
                        }
                    }
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
                s.event_hooks.push("Transcript".to_string());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                    s.event_hooks.remove(0);
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
/// Runs the full harness pipeline (observe → detect → strategize) on each event.
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
    let mut pipeline_results = Vec::new();

    for ev in &events {
        let hook_name = ev
            .get("hookName")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let session_id = ev
            .get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let agent_id = ev.get("agentId").and_then(|v| v.as_str()).unwrap_or("root");
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
            let session =
                SessionState::new(session_id.to_string(), task, agent_type.clone(), agent_type);
            let mut sessions = state.store.write().await;
            sessions.insert(session_id.to_string(), session);
            tracing::info!(
                session_id = %session_id,
                agent_class = %agent_class,
                "Batch-created session"
            );
        }

        // Build registry from session preset
        let registry = {
            let sessions = state.store.read().await;
            let base = sessions
                .get(session_id)
                .map(|s| build_registry_for_preset(&s.preset))
                .unwrap_or_else(|| build_registry_for_preset("solo"));
            let cfg = state.harness_config.read().await;
            let mut filtered = forge_harness::plugin_registry::PluginRegistry::new();
            for obs in base.observers() {
                if cfg.enabled_observers.contains(&obs.name().to_string()) {
                    filtered.register_observer(obs.clone());
                }
            }
            for det in base.detectors() {
                if cfg.enabled_detectors.contains(&det.name().to_string()) {
                    filtered.register_detector(det.clone());
                }
            }
            for strat in base.strategies() {
                if cfg.enabled_strategies.contains(&strat.name().to_string()) {
                    filtered.register_strategy(strat.clone());
                }
            }
            filtered
        };

        // Convert and run pipeline
        let agent_event = hook_to_agent_event(hook_name, agent_id, tool_name, &payload, 0);
        if let Some(event) = agent_event {
            let cycle_result = run_pipeline_cycle(
                &state,
                session_id,
                agent_id,
                hook_name,
                &registry,
                &event,
                &payload,
                starts_session,
                stops_session,
            )
            .await;
            pipeline_results.push(cycle_result);
            ingested += 1;
        }
    }

    Json(json!({
        "status": "ok",
        "agentClass": agent_class,
        "eventsReceived": events.len(),
        "eventsIngested": ingested,
        "pipeline": pipeline_results,
    }))
}

/// GET /api/v1/agents/status
///
/// Returns connection status for all supported agent types.
pub async fn agent_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let mut agents: HashMap<String, Value> = HashMap::new();

    for s in sessions.values() {
        let entry = agents.entry(s.agent_type.clone()).or_insert_with(|| {
            json!({
                "agent_type": s.agent_type,
                "session_count": 0,
                "active_sessions": 0,
                "last_seen": null,
                "status": "not_detected",
            })
        });
        entry["session_count"] = json!(entry["session_count"].as_u64().unwrap_or(0) + 1);
        if s.status == SessionStatus::Running {
            entry["active_sessions"] = json!(entry["active_sessions"].as_u64().unwrap_or(0) + 1);
        }
        entry["last_seen"] = json!(s.completed_at.unwrap_or(s.created_at).to_rfc3339());
        entry["status"] = json!("connected");
    }

    // Add known agent types that haven't been seen
    for at in &[
        "claude-code",
        "cursor",
        "antigravity",
        "langgraph",
        "crewai",
        "autogen",
        "aider",
        "cline",
        "copilot",
        "windsurf",
        "devin",
    ] {
        agents.entry(at.to_string()).or_insert_with(|| {
            json!({
                "agent_type": at,
                "session_count": 0,
                "active_sessions": 0,
                "last_seen": null,
                "status": "not_detected",
            })
        });
    }

    Json(json!({
        "agents": agents.values().collect::<Vec<_>>(),
        "total_agent_types_detected": agents.values().filter(|v| v["status"] == "connected").count(),
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

/// Auto-detect the model name from environment variables or hook metadata.
/// Called when `model_name` is still None after processing events.
fn detect_model_from_env() -> Option<String> {
    // Check common environment variables for LLM model info
    for var in &[
        "CLAUDE_MODEL",
        "ANTHROPIC_MODEL",
        "ANTHROPIC_DEFAULT_MODEL",
        "OPENAI_MODEL",
        "DEEPSEEK_MODEL",
    ] {
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                return Some(val);
            }
        }
    }
    None
}

/// Estimate token count from session data when no real transcript is available.
/// Uses heuristics: tool calls, prompt lengths, event counts.
fn estimate_tokens(s: &SessionState) -> (u64, u64) {
    // Base estimate: each tool call costs ~800 tokens (input+output)
    let total_tool_calls: u64 = s.tool_counts.values().sum();
    let mut est_input = total_tool_calls * 500; // tool definitions + context
    let mut est_output = total_tool_calls * 300; // tool results + reasoning

    // Each user prompt contributes tokens
    for prompt in &s.prompt_history {
        // Rough: 1 token per 4 chars
        est_input += (prompt.len() as u64 / 4).max(20);
    }

    // Each event adds context overhead
    est_input += s.event_count * 50;

    // Context pressure events suggest large context windows
    if !s.context_pressure_history.is_empty() {
        let avg_pressure = s.context_pressure_history.iter().sum::<f64>()
            / s.context_pressure_history.len() as f64;
        // High pressure suggests large context
        est_input += (avg_pressure * 100_000.0) as u64;
    }

    // Minimum estimates
    est_input = est_input.max(100);
    est_output = est_output.max(50);

    (est_input, est_output)
}

/// Map a detection category string to the proper IssueCategory enum variant.
/// This allows strategies to match on category-specific variants.
fn detection_category_from_str(
    cat: &str,
    desc: &str,
) -> forge_sdk::types::detection::IssueCategory {
    match cat {
        "loop" => forge_sdk::types::detection::IssueCategory::LoopDetected {
            tool_name: desc.to_string(),
            call_count: 4,
            no_progress_turns: 4,
        },
        "stale_context" => forge_sdk::types::detection::IssueCategory::StaleContext {
            file_path: desc.to_string(),
            read_count: 1,
            context_pressure: 0.9,
        },
        "cost_anomaly" => {
            forge_sdk::types::detection::IssueCategory::CostAnomaly {
                expected_cost: 0.01,
                actual_cost: 0.05,
                multiplier: 5.0,
            }
        }
        "runaway_cost" => {
            forge_sdk::types::detection::IssueCategory::RunawayCost { acceleration: 2.0 }
        }
        "secret_leak" => {
            forge_sdk::types::detection::IssueCategory::SecretLeak {
                secret_type: desc.to_string(),
            }
        }
        "accuracy_risk" => {
            forge_sdk::types::detection::IssueCategory::AccuracyRisk {
                risk_factors: vec![desc.to_string()],
            }
        }
        "compliance_gap" => {
            forge_sdk::types::detection::IssueCategory::ComplianceGap {
                gap_type: desc.to_string(),
            }
        }
        "variety_collapse" => {
            forge_sdk::types::detection::IssueCategory::VarietyCollapse {
                similarity_score: 0.95,
                agent_count: 1,
            }
        }
        "model_mismatch" => {
            forge_sdk::types::detection::IssueCategory::ModelMismatch {
                task_complexity: "high".into(),
                model_used: "unknown".into(),
                suggested_model: "claude-sonnet-4-6".into(),
            }
        }
        "deadlock" => {
            forge_sdk::types::detection::IssueCategory::Deadlock {
                agents: vec!["unknown".into()],
                wait_duration_secs: 60,
            }
        }
        "conversation_stall" => {
            forge_sdk::types::detection::IssueCategory::ConversationStall {
                duration_secs: 60,
            }
        }
        "goal_drift" => {
            forge_sdk::types::detection::IssueCategory::GoalDrift {
                similarity_to_original: 0.3,
            }
        }
        "hallucination" => {
            forge_sdk::types::detection::IssueCategory::Hallucination {
                reference: desc.to_string(),
                reference_type: "file".into(),
            }
        }
        "prompt_injection" => {
            forge_sdk::types::detection::IssueCategory::PromptInjection {
                pattern_matched: desc.to_string(),
            }
        }
        "resource_exhaustion" => {
            forge_sdk::types::detection::IssueCategory::ResourceExhaustion {
                resource: desc.to_string(),
                usage_pct: 95.0,
            }
        }
        "output_degradation" => {
            forge_sdk::types::detection::IssueCategory::OutputDegradation {
                trend_slope: -0.5,
                consecutive_declines: 3,
            }
        }
        _ => forge_sdk::types::detection::IssueCategory::AccuracyRisk {
            risk_factors: vec![format!("{}: {}", cat, desc)],
        },
    }
}

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

/// Direct event-based detection — analyzes raw event patterns to find issues.
/// This catches problems that formal detectors miss because observers produce sparse data.
fn detect_from_events(
    s: &SessionState,
    registry: &forge_harness::plugin_registry::PluginRegistry,
) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let mut detections = Vec::new();
    let mut strategies = Vec::new();
    let mut interventions = Vec::new();

    // Dedup helper: skip if same category+description already exists in session
    let is_dup = |cat: &str, desc: &str| -> bool {
        s.detections.iter().any(|d| {
            d.get("category").and_then(|c| c.as_str()) == Some(cat)
                && d.get("description").and_then(|c| c.as_str()) == Some(desc)
        })
    };

    // ── LOOP DETECTION: same tool 3+ times consecutively ──
    // Uses loop_detected_tools HashSet to ensure each tool is only reported ONCE per session.
    let mut last_tool = String::new();
    let mut repeat_count = 0u32;
    let mut loop_start_event = 0u64;
    for (i, ev) in s.events.iter().enumerate() {
        let tool = match ev {
            AgentEvent::ToolCallStart { tool, .. } | AgentEvent::ToolCallEnd { tool, .. } => {
                tool.clone()
            }
            _ => String::new(),
        };
        if !tool.is_empty() && tool == last_tool {
            if repeat_count == 0 {
                loop_start_event = i as u64;
            }
            repeat_count += 1;
        } else {
            if repeat_count >= 3 && !s.loop_detected_tools.contains(&last_tool) {
                detections.push(json!({
                    "category": "loop",
                    "severity": "Warning",
                    "confidence": 0.85,
                    "description": format!(
                        "Loop detected: '{}' called {} times consecutively (events #{}-#{}) — agent may be stuck retrying the same action. Check if the tool result is being ignored or the agent is falling into a repetitive pattern.",
                        last_tool, repeat_count + 1, loop_start_event + 1, i
                    ),
                }));
            }
            last_tool = tool;
            repeat_count = 0;
        }
    }
    // Check the last sequence too
    if repeat_count >= 3 && !s.loop_detected_tools.contains(&last_tool) {
        detections.push(json!({
            "category": "loop",
            "severity": "Warning",
            "confidence": 0.85,
            "description": format!(
                "Loop detected: '{}' called {} times consecutively (events #{}-#{}) — agent may be stuck retrying the same action.",
                last_tool, repeat_count + 1, loop_start_event + 1, s.events.len()
            ),
        }));
    }

    // ── CONTEXT PRESSURE: check if context is critically high ──
    if !s.reported_categories.contains("stale_context") {
        if let Some(&last_pressure) = s.context_pressure_history.last() {
            if last_pressure > 0.85 {
                detections.push(json!({
                    "category": "stale_context",
                    "severity": if last_pressure > 0.95 { "Critical" } else { "Warning" },
                    "confidence": last_pressure,
                    "description": format!("Context pressure at {:.0}% — agent may be losing track of earlier context. Consider compacting or starting fresh.", last_pressure * 100.0),
                }));
                strategies.push(json!({
                    "strategy": "compact",
                    "detection": "stale_context",
                    "intervention": format!("Compact: reduce context from {:.0}% to target 60%", last_pressure * 100.0),
                }));
            }
        }
    }

    // ── TOOL ERRORS: check error rate (report once per session) ──
    if !s.reported_categories.contains("accuracy_risk") {
        let total: u64 = s.tool_counts.values().sum();
        let errors: u64 = s.tool_errors.values().sum();
        if total > 0 && errors > 0 {
            let error_rate = errors as f64 / total as f64;
            if error_rate > 0.2 {
                detections.push(json!({
                    "category": "accuracy_risk",
                    "severity": "Error",
                    "confidence": error_rate,
                    "description": format!("High tool error rate: {}/{} calls failed ({:.0}%). Review tool definitions and permissions.", errors, total, error_rate * 100.0),
                }));
            }
        }
    }

    // ── COST ANOMALY: check if token usage per event is spiking (report once) ──
    if !s.reported_categories.contains("cost_anomaly") && s.event_count > 5 {
        let avg_tokens_per_event =
            (s.total_input_tokens + s.total_output_tokens) as f64 / s.event_count as f64;
        if avg_tokens_per_event > 5000.0 {
            detections.push(json!({
                "category": "cost_anomaly",
                "severity": "Warning",
                "confidence": 0.7,
                "description": format!("High token usage: {:.0} tokens/event average. Consider optimizing prompts or enabling caching.", avg_tokens_per_event),
            }));
        }
    }

    // ── MAP STRATEGIES to direct detections (simple category matching) ──
    // Formal async strategies run in the main pipeline; here we just tag detections.
    let strategy_map: &[(&str, &str)] = &[
        ("loop", "nudge"),
        ("stale_context", "compact"),
        ("accuracy_risk", "nudge"),
        ("cost_anomaly", "nudge"),
        ("secret_leak", "circuit_break"),
        ("compliance_gap", "quarantine"),
    ];
    for det in &detections {
        let cat = det["category"].as_str().unwrap_or("");
        let desc = det["description"].as_str().unwrap_or("");
        let strategy_name = strategy_map.iter()
            .find(|(c, _)| *c == cat)
            .map(|(_, s)| *s)
            .unwrap_or("nudge");
        interventions.push(json!({
            "strategy": strategy_name,
            "action": desc,
        }));
    }

    // ── SECRET LEAK: scan tool content for secrets ──
    let secret_patterns = [
        // Only match credential patterns in suspicious contexts (assignments, configs, etc.)
        // to avoid false positives from variable names in source code
        ("sk-", "OpenAI API key"),
        ("api_key=", "API key assignment"),
        ("api_key:", "API key in config"),
        ("Bearer ", "Bearer token in header"),
        ("password=", "Password assignment"),
        ("password:", "Password in config"),
        ("passwd=", "Password in config"),
        ("-----BEGIN RSA PRIVATE KEY", "Private key"),
        ("-----BEGIN OPENSSH PRIVATE KEY", "SSH private key"),
        ("ghp_", "GitHub PAT"),
        ("xoxb-", "Slack bot token"),
        ("xoxp-", "Slack user token"),
        ("AKIA", "AWS access key"),
        ("secret_key=", "Secret key assignment"),
        ("secret_key:", "Secret key in config"),
        ("access_token=", "Access token in output"),
    ];
    for ev in s.events.iter().rev().take(20) {
        if let Ok(v) = serde_json::to_value(ev) {
            if let Some(content) = v
                .get("result")
                .and_then(|r| r.get("content"))
                .and_then(|c| c.as_str())
            {
                for (pattern, name) in &secret_patterns {
                    if content.to_lowercase().contains(pattern) && !is_dup("secret_leak", pattern) {
                        detections.push(json!({
                            "category": "secret_leak",
                            "severity": "Critical",
                            "confidence": 0.95,
                            "description": format!("Potential {} detected in tool output. Immediate action required: rotate the exposed credential and remove it from output.", name),
                        }));
                        break;
                    }
                }
            }
        }
    }

    (detections, strategies, interventions)
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

/// Flatten an AgentEvent into a JSON observation with ALL fields that
/// downstream detectors expect. This bridges the gap between observer output
/// (computed metrics) and detector input (raw event context).
fn event_to_observation(event: &AgentEvent) -> Option<Value> {
    match event {
        AgentEvent::ToolCallStart { tool, args, timestamp, .. } => {
            let file_path = args.get("file_path").and_then(|v| v.as_str());
            let command = args.get("command").and_then(|v| v.as_str());
            Some(json!({
                "tool_name": tool,
                "tool": tool,
                "file_path": file_path,
                "file_reference": file_path,
                "command": command,
                "content": command.unwrap_or(""),
                "msg_timestamp_ms": timestamp.timestamp_millis(),
                "resource": "tool_call",
                "usage_pct": 0.0,
            }))
        }
        AgentEvent::ToolCallEnd { tool, result, timestamp, .. } => Some(json!({
            "tool_name": tool,
            "tool": tool,
            "content": result.content,
            "output": result.content,
            "quality_score": if result.is_error { 0.3 } else { 0.9 },
            "msg_timestamp_ms": timestamp.timestamp_millis(),
            "cost_per_turn": result.token_count as f64 * 0.000003,  // rough estimate
        })),
        AgentEvent::MessageSent { from, content, timestamp, .. } => {
            let text = match content {
                forge_sdk::events::MessageContent::Text(t) => t.clone(),
                _ => return None,
            };
            Some(json!({
                "content": text,
                "original_task": text,
                "current_output": text,
                "agent_output": text,
                "msg_timestamp_ms": timestamp.timestamp_millis(),
            }))
        }
        AgentEvent::ContextPressure { current_ratio, timestamp, .. } => Some(json!({
            "context_pressure": current_ratio,
            "context_ratio": current_ratio,
            "usage_pct": current_ratio * 100.0,
            "resource": "context_window",
            "msg_timestamp_ms": timestamp.timestamp_millis(),
        })),
        AgentEvent::TokenUsage { model, input, output, timestamp, .. } => Some(json!({
            "model": model,
            "task": format!("{} tokens used", input + output),
            "msg_timestamp_ms": timestamp.timestamp_millis(),
            "cost_per_turn": (*input as f64 * 3.0 + *output as f64 * 15.0) / 1_000_000.0,
            "quality_score": if *output > 0 { 0.8 } else { 0.5 },
        })),
        AgentEvent::Forked { child_id, task, timestamp, .. } => Some(json!({
            "waiting_agent": child_id,
            "waiting_for": task,
            "wait_duration_secs": 0u64,
            "msg_timestamp_ms": timestamp.timestamp_millis(),
            "resource": "subagent",
        })),
        AgentEvent::OutputDelta { text, timestamp, .. } => {
            if text.is_empty() { return None; }
            Some(json!({
                "agent_output": text,
                "content": text,
                "current_output": text,
                "msg_timestamp_ms": timestamp.timestamp_millis(),
                "quality_score": 0.7,
            }))
        }
        AgentEvent::Started { task, .. } => Some(json!({
            "original_task": task,
            "task": task,
            "content": task,
        })),
        AgentEvent::Completed { summary, .. } => Some(json!({
            "compliance_gap": if summary.contains("error") { "Errors in session" } else { "" },
            "current_output": summary,
        })),
        AgentEvent::Failed { error, .. } => Some(json!({
            "compliance_gap": error,
            "current_output": error,
            "quality_score": 0.0,
        })),
        _ => None,
    }
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
            // Extract tool result and args from Claude Code's hook payload
            let tool_response = payload.get("tool_response");
            let raw_content = tool_response.and_then(|v| v.as_str()).unwrap_or("");
            let ti = payload.get("tool_input").cloned().unwrap_or(json!({}));

            // Build synthetic description from tool_input (shows what was requested)
            let synthetic = match tool_name {
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
                    format!("Command: {}", if cmd.len() > 100 { &cmd[..100] } else { cmd })
                }
                "Grep" => {
                    let pattern = ti.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                    format!("Searched: {}", pattern)
                }
                _ => format!("{} called", tool_name),
            };

            // Combine: show what was requested + what happened
            let content = if raw_content.is_empty() {
                synthetic
            } else {
                format!("{}\nResult: {}", synthetic, raw_content)
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
                    duration_ms: payload
                        .get("duration_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
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
                duration_ms: payload
                    .get("duration_ms")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0),
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
