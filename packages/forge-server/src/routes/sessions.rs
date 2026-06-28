//! Session endpoints — list, get, delete, pause, resume.
//!
//! Sessions are created automatically by the ingest API when real
//! agent events arrive. There is no manual session creation.
//! No MockAgent. Real agents only.

use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use forge_sdk::events::Intervention;
use serde_json::{json, Value};
use std::sync::Arc;

/// Sessions are auto-created via POST /api/v1/ingest/event.
/// This endpoint returns a message directing users to that flow.
pub async fn create() -> Json<Value> {
    Json(json!({
        "error": "Manual session creation is disabled.",
        "message": "Sessions are created automatically when you use your AI agent.",
        "how": "Just use Claude Code, LangGraph, CrewAI, or any agent normally.",
        "dashboard": "Open http://127.0.0.1:3000 to see auto-detected sessions."
    }))
}

/// GET /v1/sessions
/// Returns all sessions with their status, metadata, and pipeline summary.
pub async fn list(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let mut items: Vec<Value> = sessions
        .iter()
        .map(|(id, s)| {
            json!({
                "id": id,
                "task": s.task,
                "agent_type": s.agent_type,
                "preset": s.preset,
                "status": s.status.to_string(),
                "created_at": s.created_at.to_rfc3339(),
                "completed_at": s.completed_at.map(|t| t.to_rfc3339()),
                "health_score": s.health_score,
                "result": s.result,
                "pipeline": {
                    "event_count": s.event_count,
                    "observation_count": s.observations.len(),
                    "detection_count": s.detections.len(),
                    "intervention_count": s.interventions.len(),
                }
            })
        })
        .collect();
    items.sort_by(|a, b| {
        b.get("created_at")
            .and_then(|v| v.as_str())
            .cmp(&a.get("created_at").and_then(|v| v.as_str()))
    });
    Json(json!({"sessions": items, "total": items.len()}))
}

/// GET /v1/sessions/:id
/// Returns full session detail including pipeline data.
pub async fn get(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(s) => Json(json!({
            "id": s.id,
            "task": s.task,
            "agent_type": s.agent_type,
            "preset": s.preset,
            "status": s.status.to_string(),
            "created_at": s.created_at.to_rfc3339(),
            "completed_at": s.completed_at.map(|t| t.to_rfc3339()),
            "health_score": s.health_score,
            "result": s.result,
            "event_count": s.events.len(),
            "pipeline": {
                "observations": s.observations,
                "detections": s.detections,
                "strategy_results": s.strategy_results,
                "interventions": s.interventions,
                "total_observations": s.observations.len(),
                "total_detections": s.detections.len(),
                "total_interventions": s.interventions.len(),
            }
        })),
        None => Json(json!({"error": "session not found", "id": id})),
    }
}

/// DELETE /v1/sessions/:id
pub async fn delete(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let mut sessions = state.store.write().await;
    if let Some(session) = sessions.get(&id) {
        let _ = session.cancel_tx.send(true);
    }
    sessions.remove(&id);
    Json(json!({"deleted": id}))
}

/// POST /v1/sessions/:id/pause
pub async fn pause(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(session) => {
            let iv = Intervention::Pause {
                reason: "user requested pause".into(),
                checkpoint_id: uuid::Uuid::new_v4(),
            };
            match session.intervention_tx.try_send(iv) {
                Ok(_) => Json(json!({"paused": id, "status": "ok"})),
                Err(_) => Json(json!({"paused": id, "status": "channel_full"})),
            }
        }
        None => Json(json!({"error": "session not found"})),
    }
}

/// POST /v1/sessions/:id/resume
pub async fn resume(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(session) => match session.intervention_tx.try_send(Intervention::Resume) {
            Ok(_) => Json(json!({"resumed": id, "status": "ok"})),
            Err(_) => Json(json!({"resumed": id, "status": "channel_full"})),
        },
        None => Json(json!({"error": "session not found"})),
    }
}

/// GET /v1/sessions/:id/analysis
/// Returns a complete, human-readable analysis of the session including:
/// token breakdown, tool usage, context health, loop detection, degradation,
/// health scores with explanations, stop reason, and recommendations.
pub async fn analysis(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    let s = match sessions.get(&id) {
        Some(s) => s,
        None => return Json(json!({"error": "session not found", "id": id})),
    };

    let total_tokens = s.total_input_tokens + s.total_output_tokens;
    let _cache_total = s.total_cache_read + s.total_cache_write;
    let cache_hit_pct = if s.total_input_tokens > 0 {
        (s.total_cache_read as f64 / s.total_input_tokens as f64) * 100.0
    } else {
        0.0
    };

    // Cost estimate based on detected model
    let model = s.model_name.as_deref().unwrap_or("unknown");
    let (input_price, output_price, model_family) = model_pricing(model);
    let est_cost =
        (s.total_input_tokens as f64 * input_price) + (s.total_output_tokens as f64 * output_price);
    let cache_savings = s.total_cache_read as f64 * input_price * 0.9; // cache reads save 90% of input cost
    let effective_cost = est_cost - cache_savings;

    // Tool usage analysis
    let total_tool_calls: u64 = s.tool_counts.values().sum();
    let total_tool_errors: u64 = s.tool_errors.values().sum();
    let mut tool_breakdown: Vec<Value> = s.tool_counts.iter()
        .map(|(name, count)| {
            let errors = s.tool_errors.get(name).copied().unwrap_or(0);
            let pct = if total_tool_calls > 0 { (*count as f64 / total_tool_calls as f64) * 100.0 } else { 0.0 };
            json!({
                "tool": name,
                "calls": count,
                "errors": errors,
                "error_rate_pct": if *count > 0 { (errors as f64 / *count as f64) * 100.0 } else { 0.0 },
                "pct_of_total": pct,
            })
        })
        .collect();
    tool_breakdown.sort_by(|a, b| b["calls"].as_u64().cmp(&a["calls"].as_u64()));

    // Context pressure analysis
    let context_peaks = s.context_pressure_history.len();
    let context_avg = if context_peaks > 0 {
        s.context_pressure_history.iter().sum::<f64>() / context_peaks as f64
    } else {
        0.0
    };
    let context_max = s
        .context_pressure_history
        .iter()
        .cloned()
        .fold(0.0f64, f64::max);
    let compaction_events = s.context_pressure_history.len();

    // Health analysis with explanations
    let hs = s.health_score.as_ref();
    let health_verdict = match hs {
        Some(h) if h.overall > 0.8 => "Healthy — agent performing well across all dimensions",
        Some(h) if h.overall > 0.6 => "Moderate — some dimensions need attention",
        Some(h) if h.overall > 0.4 => "Degraded — multiple issues detected",
        Some(_) => "Critical — agent requires immediate intervention",
        None => "No health data yet — run more events to build analysis",
    };

    // Stop reason analysis
    let stop_analysis = match (&s.stop_reason, &s.status) {
        (Some(reason), crate::session::store::SessionStatus::Completed) => {
            format!("✅ Session completed normally. {}", reason)
        }
        (Some(reason), crate::session::store::SessionStatus::Failed) => {
            format!("❌ Session failed: {}", reason)
        }
        (None, crate::session::store::SessionStatus::Running) => {
            "🟢 Session is still running — no stop reason yet".to_string()
        }
        (None, _) => "Session ended without recording a stop reason".to_string(),
        _ => format!("Session status: {:?}", s.status),
    };

    // Duration
    let duration_secs = s
        .completed_at
        .map(|t| (t - s.created_at).num_seconds())
        .unwrap_or_else(|| (Utc::now() - s.created_at).num_seconds());

    // Generate recommendations
    let mut recommendations: Vec<&str> = Vec::new();
    if cache_hit_pct < 20.0 && total_tokens > 1000 {
        recommendations
            .push("Low cache hit rate — consider enabling prompt caching to reduce costs");
    }
    if total_tool_errors > 0 && total_tool_errors as f64 / total_tool_calls.max(1) as f64 > 0.1 {
        recommendations.push("High tool error rate — review tool definitions and error handling");
    }
    if context_avg > 0.75 {
        recommendations.push("Context pressure consistently high — compact more aggressively or reduce conversation length");
    }
    if s.subagent_count > 5 {
        recommendations
            .push("High subagent spawn count — consider consolidating tasks to reduce overhead");
    }
    if s.event_count > 100 && s.detections.is_empty() {
        recommendations.push("No issues detected across many events — your agent is running well");
    }
    if !s.degradation_warnings.is_empty() {
        recommendations.push("Output degradation detected — review agent prompts and tool outputs");
    }

    Json(json!({
        "session_id": s.id,
        "task": s.task,
        "agent_type": s.agent_type,
        "status": s.status.to_string(),
        "duration_secs": duration_secs,
        "model": s.model_name,
        "stop_analysis": stop_analysis,
        "health_verdict": health_verdict,

        "token_analysis": {
            "total_tokens": total_tokens,
            "input_tokens": s.total_input_tokens,
            "output_tokens": s.total_output_tokens,
            "cache_read_tokens": s.total_cache_read,
            "cache_write_tokens": s.total_cache_write,
            "cache_hit_pct": cache_hit_pct,
            "estimated_cost_usd": effective_cost,
            "gross_cost_usd": est_cost,
            "cache_savings_usd": cache_savings,
            "model_family": model_family,
            "input_price_per_m": input_price * 1_000_000.0,
            "output_price_per_m": output_price * 1_000_000.0,
            "token_efficiency": if total_tokens > 0 {
                (s.total_output_tokens as f64 / total_tokens.max(1) as f64) * 100.0
            } else { 0.0 },
        },

        "tool_analysis": {
            "total_calls": total_tool_calls,
            "total_errors": total_tool_errors,
            "error_rate_pct": if total_tool_calls > 0 {
                (total_tool_errors as f64 / total_tool_calls as f64) * 100.0
            } else { 0.0 },
            "unique_tools": s.tool_counts.len(),
            "total_duration_ms": s.total_tool_ms,
            "breakdown": tool_breakdown,
        },

        "context_analysis": {
            "compaction_events": compaction_events,
            "avg_pressure_pct": context_avg * 100.0,
            "max_pressure_pct": context_max * 100.0,
            "status": if context_max > 0.85 { "critical" } else if context_avg > 0.6 { "warning" } else { "healthy" },
        },

        "loop_analysis": {
            "patterns_detected": s.loop_patterns.len(),
            "patterns": s.loop_patterns,
        },

        "degradation_analysis": {
            "warnings": s.degradation_warnings.len(),
            "details": s.degradation_warnings,
        },

        "health_analysis": {
            "overall": hs.map(|h| h.overall).unwrap_or(1.0),
            "verdict": health_verdict,
            "dimensions": hs.map(|h| json!({
                "token_efficiency": h.dimensions.token_efficiency,
                "latency": h.dimensions.latency,
                "cost": h.dimensions.cost,
                "accuracy": h.dimensions.accuracy,
                "orchestration": h.dimensions.orchestration,
                "security": h.dimensions.security,
                "reliability": h.dimensions.reliability,
                "context_quality": h.dimensions.context_quality,
                "compliance": h.dimensions.compliance,
            })),
        },

        "session_summary": {
            "total_events": s.event_count,
            "user_prompts": s.user_prompt_count,
            "subagents_spawned": s.subagent_count,
            "detections": s.detections.len(),
            "interventions": s.interventions.len(),
            "observations": s.observations.len(),
        },

        // Observation details grouped by dimension
        "observation_details": group_observations_by_dimension(&s.observations),

        // Raw detection and intervention data for deep inspection
        "detection_details": s.detections.clone(),
        "intervention_details": s.interventions.clone(),

        "recommendations": recommendations,
    }))
}

use chrono::Utc;

/// Group observations by their dimension field, returning counts and latest samples.
fn group_observations_by_dimension(observations: &[Value]) -> Value {
    use std::collections::HashMap;
    let mut groups: HashMap<String, Vec<&Value>> = HashMap::new();
    for obs in observations {
        if let Some(dim) = obs.get("dimension").and_then(|v| v.as_str()) {
            groups.entry(dim.to_string()).or_default().push(obs);
        }
    }
    let result: Vec<Value> = groups
        .into_iter()
        .map(|(dim, items)| {
            let count = items.len();
            let latest = items.last().cloned();
            json!({
                "dimension": dim,
                "count": count,
                "latest": latest,
                "samples": items.iter().rev().take(5).cloned().collect::<Vec<_>>(),
            })
        })
        .collect();
    json!(result)
}

/// Return (input_price_per_token, output_price_per_token, model_family_name) for a model.
fn model_pricing(model: &str) -> (f64, f64, String) {
    let m = model.to_lowercase();
    // Claude models
    if m.contains("opus") {
        return (0.000015, 0.000075, "Claude Opus".into());
    }
    if m.contains("sonnet") {
        return (0.000003, 0.000015, "Claude Sonnet".into());
    }
    if m.contains("haiku") {
        return (0.0000008, 0.000004, "Claude Haiku".into());
    }
    if m.contains("fable") {
        return (0.000003, 0.000015, "Claude Fable".into());
    }
    if m.contains("claude") {
        return (0.000003, 0.000015, "Claude (default)".into());
    }
    // GPT models
    if m.contains("gpt-4o") {
        return (0.0000025, 0.000010, "GPT-4o".into());
    }
    if m.contains("gpt-4") {
        return (0.000030, 0.000060, "GPT-4".into());
    }
    if m.contains("gpt-3.5") {
        return (0.0000005, 0.0000015, "GPT-3.5".into());
    }
    if m.contains("o1") || m.contains("o3") {
        return (0.000015, 0.000060, "OpenAI o-series".into());
    }
    if m.contains("gpt") {
        return (0.0000025, 0.000010, "GPT (default)".into());
    }
    // DeepSeek
    if m.contains("deepseek") {
        return (0.00000027, 0.0000011, "DeepSeek".into());
    }
    // Gemini
    if m.contains("gemini") {
        return (0.00000125, 0.000005, "Gemini".into());
    }
    // Default / unknown
    (0.000003, 0.000015, "unknown (using default)".into())
}

/// GET /v1/sessions/:id/checkpoints
pub async fn checkpoints(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    Json(json!({"checkpoints": [], "message": "Checkpoints not yet persisted to API"}))
}
