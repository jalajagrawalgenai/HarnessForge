use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

pub async fn get_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(s) => {
            let events: Vec<Value> = s
                .events
                .iter()
                .map(|e| serde_json::to_value(e).unwrap_or(json!({})))
                .collect();
            Json(json!({"session_id":id,"events":events,"count":events.len(),"filters":params}))
        }
        None => Json(json!({"session_id":id,"events":[],"error":"session not found"})),
    }
}

pub async fn get_report(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(s) => Json(json!({
            "session_id":id,"task":s.task,"agent_type":s.agent_type,
            "status":s.status.to_string(),"events_count":s.events.len(),
            "result":s.result,
            "harness_effectiveness": if s.result.as_ref().map(|r|r.intervention_count>0).unwrap_or(false) {"active"} else {"passive"}
        })),
        None => Json(json!({"error":"session not found"})),
    }
}

pub async fn export_audit(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Value> {
    let fmt = params.get("format").map(|s| s.as_str()).unwrap_or("json");
    let sessions = state.store.read().await;
    let count = sessions.get(&id).map(|s| s.events.len()).unwrap_or(0);
    Json(json!({"session_id":id,"format":fmt,"events":count,"exported":true}))
}

pub async fn replay(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let sessions = state.store.read().await;
    match sessions.get(&id) {
        Some(s) => {
            let events: Vec<Value> = s
                .events
                .iter()
                .map(|e| serde_json::to_value(e).unwrap_or(json!({})))
                .collect();
            Json(json!({"session_id":id,"timeline":events,"bookmarks":[]}))
        }
        None => Json(json!({"error":"session not found"})),
    }
}
