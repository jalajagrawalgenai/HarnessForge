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
/// Returns all sessions with their status and metadata.
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

/// GET /v1/sessions/:id/checkpoints
pub async fn checkpoints(
    State(_state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Json<Value> {
    Json(json!({"checkpoints": [], "message": "Checkpoints not yet persisted to API"}))
}
