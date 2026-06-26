use crate::session::store::SessionStatus;
use crate::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

pub async fn overview(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let total = sessions.len() as u64;
    let completed = sessions
        .values()
        .filter(|s| s.status == SessionStatus::Completed)
        .count() as u64;
    let running = sessions
        .values()
        .filter(|s| s.status == SessionStatus::Running)
        .count() as u64;
    Json(
        json!({"total_sessions":total,"completed":completed,"running":running,"total_tokens":0,"total_cost":0.0,"avg_health":1.0,"interventions_total":0}),
    )
}
pub async fn tokens(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let items: Vec<Value> = sessions
        .iter()
        .map(|(id, s)| json!({"session_id":id,"status":s.status.to_string(),"tokens":0}))
        .collect();
    Json(json!({"tokens":items}))
}
pub async fn costs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let items: Vec<Value> = sessions
        .iter()
        .map(|(id, s)| json!({"session_id":id,"status":s.status.to_string(),"cost":0.0}))
        .collect();
    Json(json!({"costs":items}))
}
pub async fn interventions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let total: u64 = sessions
        .values()
        .map(|s| s.result.as_ref().map(|r| r.intervention_count).unwrap_or(0))
        .sum();
    Json(json!({"interventions":[],"total":total}))
}
pub async fn health(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let items: Vec<Value> = sessions
        .iter()
        .filter_map(|(id, s)| {
            s.health_score.as_ref().map(
                |h| json!({"session_id":id,"overall":h.overall,"trend":format!("{:?}",h.trend)}),
            )
        })
        .collect();
    Json(json!({"health_trend":items}))
}
