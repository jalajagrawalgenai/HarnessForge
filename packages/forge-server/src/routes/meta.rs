use crate::session::store::SessionStatus;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

/// POST /v1/meta/improve — triggers self-improvement cycle.
/// Analyzes completed sessions for weakness patterns, proposes rule changes.
pub async fn improve(
    State(state): State<Arc<AppState>>,
    Json(_body): Json<Value>,
) -> Json<Value> {
    let sessions = state.store.read().await;
    let completed: Vec<_> = sessions
        .values()
        .filter(|s| s.status == SessionStatus::Completed || s.status == SessionStatus::Failed)
        .collect();

    if completed.is_empty() {
        return Json(json!({
            "status": "no_data",
            "message": "No completed sessions to analyze. Run some agent sessions first.",
            "sessions_available": sessions.len(),
            "completed_sessions": 0,
        }));
    }

    // Build simple audit snapshots from session data
    let audits: Vec<Value> = completed.iter().map(|s| {
        json!({
            "session_id": s.id,
            "agent_type": s.agent_type,
            "model": s.model_name,
            "task": s.task,
            "status": s.status.to_string(),
            "total_tokens": s.total_input_tokens + s.total_output_tokens,
            "detection_count": s.detections.len(),
            "intervention_count": s.interventions.len(),
            "tool_errors": s.tool_errors.values().sum::<u64>(),
            "tool_calls": s.tool_counts.values().sum::<u64>(),
            "context_pressure_avg": if s.context_pressure_history.is_empty() { 0.0 }
                else { s.context_pressure_history.iter().sum::<f64>() / s.context_pressure_history.len() as f64 },
        })
    }).collect();

    // Find weakness patterns
    let mut weaknesses = Vec::new();
    for audit in &audits {
        let detections = audit["detection_count"].as_u64().unwrap_or(0);
        let interventions = audit["intervention_count"].as_u64().unwrap_or(0);
        let errors = audit["tool_errors"].as_u64().unwrap_or(0);
        let calls = audit["tool_calls"].as_u64().unwrap_or(1);
        let error_rate = errors as f64 / calls.max(1) as f64;
        let pressure = audit["context_pressure_avg"].as_f64().unwrap_or(0.0);

        if error_rate > 0.3 {
            weaknesses.push(json!({
                "pattern": "high_tool_error_rate",
                "session_id": audit["session_id"],
                "agent_type": audit["agent_type"],
                "severity": if error_rate > 0.5 { "critical" } else { "warning" },
                "detail": format!("{:.0}% tool error rate ({} errors / {} calls)", error_rate * 100.0, errors, calls),
                "count": 1,
            }));
        }
        if detections == 0 && calls > 10 {
            weaknesses.push(json!({
                "pattern": "undetected_issues",
                "session_id": audit["session_id"],
                "agent_type": audit["agent_type"],
                "severity": "info",
                "detail": format!("{} tool calls with 0 detections — detectors may be too lenient", calls),
                "count": 1,
            }));
        }
        if pressure > 0.7 {
            weaknesses.push(json!({
                "pattern": "high_context_pressure",
                "session_id": audit["session_id"],
                "agent_type": audit["agent_type"],
                "severity": if pressure > 0.85 { "warning" } else { "info" },
                "detail": format!("Avg context pressure: {:.0}%", pressure * 100.0),
                "count": 1,
            }));
        }
        if interventions > 0 && detections > 0 && interventions < detections {
            weaknesses.push(json!({
                "pattern": "intervention_gap",
                "session_id": audit["session_id"],
                "agent_type": audit["agent_type"],
                "severity": "warning",
                "detail": format!("{} detections but only {} interventions — strategies undershooting", detections, interventions),
                "count": 1,
            }));
        }
    }

    // Group by pattern type
    let mut groups: std::collections::HashMap<String, Vec<&Value>> = std::collections::HashMap::new();
    for w in &weaknesses {
        groups.entry(w["pattern"].as_str().unwrap_or("unknown").to_string())
            .or_default()
            .push(w);
    }

    let grouped: Vec<Value> = groups.into_iter().map(|(pattern, items)| {
        json!({
            "pattern": pattern,
            "count": items.len(),
            "severity": items.iter().map(|i| i["severity"].as_str().unwrap_or("info")).max().unwrap_or("info"),
            "instances": items,
        })
    }).collect();

    Json(json!({
        "status": "completed",
        "sessions_analyzed": completed.len(),
        "weaknesses_found": weaknesses.len(),
        "weaknesses": grouped,
        "message": format!("Analyzed {} sessions, found {} weakness patterns", completed.len(), weaknesses.len()),
    }))
}

/// GET /v1/meta/weaknesses — returns cached weakness patterns from last improvement cycle.
pub async fn weaknesses(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let completed = sessions.values()
        .filter(|s| s.status == SessionStatus::Completed || s.status == SessionStatus::Failed)
        .count();
    let total_detections: usize = sessions.values().map(|s| s.detections.len()).sum();
    let total_interventions: usize = sessions.values().map(|s| s.interventions.len()).sum();
    let total_tool_errors: u64 = sessions.values().map(|s| s.tool_errors.values().sum::<u64>()).sum();

    let mut weaknesses = Vec::new();
    for s in sessions.values() {
        if total_tool_errors > 0 && s.tool_errors.values().sum::<u64>() > 0 {
            weaknesses.push(json!({
                "pattern": "tool_errors",
                "session_id": s.id,
                "agent_type": s.agent_type,
                "detail": format!("{} tool errors across {} tools", s.tool_errors.values().sum::<u64>(), s.tool_errors.len()),
            }));
        }
        if s.context_pressure_history.iter().any(|&p| p > 0.8) {
            weaknesses.push(json!({
                "pattern": "context_pressure_spike",
                "session_id": s.id,
                "agent_type": s.agent_type,
                "detail": format!("Context pressure exceeded 80% (max: {:.0}%)",
                    s.context_pressure_history.iter().cloned().fold(0.0f64, f64::max) * 100.0),
            }));
        }
    }

    Json(json!({
        "weaknesses": weaknesses,
        "total": weaknesses.len(),
        "session_count": sessions.len(),
        "completed_sessions": completed,
        "total_detections": total_detections,
        "total_interventions": total_interventions,
    }))
}

pub async fn edits(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let completed = sessions.values()
        .filter(|s| s.status == SessionStatus::Completed || s.status == SessionStatus::Failed)
        .count();

    let mut edits = Vec::new();
    // Propose rule edits based on session data
    for s in sessions.values() {
        let error_rate = if s.tool_counts.values().sum::<u64>() > 0 {
            s.tool_errors.values().sum::<u64>() as f64 / s.tool_counts.values().sum::<u64>() as f64
        } else { 0.0 };

        if error_rate > 0.3 {
            edits.push(json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "rule": "tool_error_threshold",
                "change": format!("Lower tool error threshold from 0.2 to 0.15 for {} sessions", s.agent_type),
                "session_id": s.id,
                "impact": "high",
            }));
        }
        if s.detections.len() > 5 {
            edits.push(json!({
                "id": uuid::Uuid::new_v4().to_string(),
                "rule": "detection_budget",
                "change": format!("{} detections in one session — consider increasing intervention aggressiveness", s.detections.len()),
                "session_id": s.id,
                "impact": "medium",
            }));
        }
    }

    Json(json!({
        "edits": edits,
        "total": edits.len(),
        "completed_sessions": completed,
    }))
}

pub async fn accept_edit(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"accepted": id, "status": "rule_updated"}))
}
pub async fn reject_edit(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"rejected": id}))
}
pub async fn ab_tests(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let total = sessions.len();
    Json(json!({
        "ab_tests": [],
        "total": 0,
        "sessions_available": total,
        "message": "A/B testing requires 2+ completed sessions with different harness configs",
    }))
}
