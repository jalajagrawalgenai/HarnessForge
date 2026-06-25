use axum::{extract::{Path, Query}, Json};
use serde_json::{json, Value};
use std::collections::HashMap;

pub async fn get_audit(Path(id): Path<String>, Query(params): Query<HashMap<String,String>>) -> Json<Value> {
    Json(json!({"session_id":id,"events":[],"filters":params}))
}
pub async fn get_report(Path(id): Path<String>) -> Json<Value> { Json(json!({"report":format!("Audit report for {}",id)})) }
pub async fn export_audit(Path(id): Path<String>, Query(params): Query<HashMap<String,String>>) -> Json<Value> {
    let fmt = params.get("format").map(|s| s.as_str()).unwrap_or("json");
    Json(json!({"export":format!("Exported {} as {}",id,fmt)}))
}
pub async fn replay(Path(id): Path<String>) -> Json<Value> { Json(json!({"replay":id,"events":[]})) }
