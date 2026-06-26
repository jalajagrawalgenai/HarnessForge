use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

pub async fn improve(Json(_body): Json<Value>) -> Json<Value> {
    Json(json!({"status":"not_enough_data","message":"Meta-harness requires 20+ completed sessions","sessions_needed":20,"current_sessions":0}))
}
pub async fn weaknesses() -> Json<Value> {
    Json(json!({"weaknesses":[],"total":0}))
}
pub async fn edits() -> Json<Value> {
    Json(json!({"edits":[],"total":0}))
}
pub async fn accept_edit(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"accepted":id}))
}
pub async fn reject_edit(Path(id): Path<String>) -> Json<Value> {
    Json(json!({"rejected":id}))
}
pub async fn ab_tests() -> Json<Value> {
    Json(json!({"ab_tests":[],"total":0}))
}
