use axum::Json;
use serde_json::{json, Value};

pub async fn get() -> Json<Value> { Json(json!({"harness":"v1.0.0","observers":[],"detectors":[],"strategies":[]})) }
pub async fn update(Json(body): Json<Value>) -> Json<Value> { Json(json!({"updated":true,"config":body})) }
pub async fn versions() -> Json<Value> { Json(json!({"versions":["v1.0.0"]})) }
