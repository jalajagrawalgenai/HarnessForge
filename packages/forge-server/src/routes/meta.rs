use axum::Json;
use serde_json::{json, Value};

pub async fn improve() -> Json<Value> {
    Json(json!({"status":"improvement_cycle_started"}))
}
pub async fn weaknesses() -> Json<Value> {
    Json(json!({"weaknesses":[]}))
}
pub async fn edits() -> Json<Value> {
    Json(json!({"edits":[]}))
}
