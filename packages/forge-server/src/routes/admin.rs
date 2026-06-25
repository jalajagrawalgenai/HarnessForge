use axum::Json;
use serde_json::{json, Value};

pub async fn list_keys() -> Json<Value> {
    Json(json!({"keys":[]}))
}
pub async fn create_key() -> Json<Value> {
    Json(json!({"key":"fk_","created":true}))
}
pub async fn quotas() -> Json<Value> {
    Json(json!({"quotas":[]}))
}
