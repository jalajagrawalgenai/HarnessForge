use axum::Json;
use serde_json::{json, Value};

pub async fn overview() -> Json<Value> { Json(json!({"sessions_today":0,"total_tokens":0,"total_cost":0.0,"avg_health":1.0})) }
pub async fn tokens() -> Json<Value> { Json(json!({"tokens":[]})) }
pub async fn costs() -> Json<Value> { Json(json!({"costs":[]})) }
pub async fn interventions() -> Json<Value> { Json(json!({"interventions":[]})) }
pub async fn health() -> Json<Value> { Json(json!({"health_trend":[]})) }
