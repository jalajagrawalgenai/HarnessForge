use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

pub async fn list_configs() -> Json<Value> {
    Json(json!({"configs":[
        {"target":"langfuse","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"wandb","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"opentelemetry","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"pagerduty","enabled":false,"endpoint":"","batch_size":1,"flush_interval_secs":5},
        {"target":"opsgenie","enabled":false,"endpoint":"","batch_size":1,"flush_interval_secs":5},
        {"target":"splunk","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"elasticsearch","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"datadog","enabled":false,"endpoint":"","batch_size":100,"flush_interval_secs":30},
        {"target":"slack","enabled":false,"endpoint":"","batch_size":1,"flush_interval_secs":5},
        {"target":"discord","enabled":false,"endpoint":"","batch_size":1,"flush_interval_secs":5}
    ]}))
}
pub async fn update_config(Path(target): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"updated":target,"config":body}))
}
pub async fn test_connection(Path(target): Path<String>) -> Json<Value> {
    Json(json!({"target":target,"connected":false,"message":"API key not configured"}))
}
pub async fn trigger_export(Json(body): Json<Value>) -> Json<Value> {
    let session_id = body
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    Json(json!({"exported":session_id,"status":"queued"}))
}
pub async fn get_alert_config() -> Json<Value> {
    Json(
        json!({"pagerduty":{"routing_key":"","enabled":false},"opsgenie":{"routing_key":"","enabled":false}}),
    )
}
pub async fn update_alert_config(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"updated":true,"alert_config":body}))
}
