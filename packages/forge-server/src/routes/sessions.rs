//! Session CRUD endpoints — create, list, get, delete, pause, resume sessions.

use crate::session::manager;
use crate::session::store::SessionState;
use crate::AppState;
use axum::extract::{Path, State};
use axum::Json;
use forge_sdk::events::Intervention;
use serde_json::{json, Value};
use std::sync::Arc;

/// POST /v1/sessions
/// Request: { "task": "...", "agent_type": "...", "preset": "..." }
/// Creates a new session and spawns the harness in the background.
pub async fn create(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let task = body
        .get("task")
        .and_then(|v| v.as_str())
        .unwrap_or("default task");
    let agent_type = body
        .get("agent_type")
        .and_then(|v| v.as_str())
        .unwrap_or("solo");
    let preset = body
        .get("preset")
        .and_then(|v| v.as_str())
        .unwrap_or("solo");

    let id = uuid::Uuid::new_v4().to_string();
    let session = SessionState::new(
        id.clone(),
        task.to_string(),
        agent_type.to_string(),
        preset.to_string(),
    );

    {
        let mut sessions = state.store.write().await;
        sessions.insert(id.clone(), session);
    }

    // Spawn harness in background
    manager::spawn_session(
        state.store.clone(),
        id.clone(),
        task.to_string(),
        agent_type.to_string(),
        preset.to_string(),
    )
    .await;

    Json(json!({
        "id": id,
        "status": "running",
        "task": task,
        "agent_type": agent_type,
        "preset": preset,
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
