use axum::{extract::Path, Json};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn list() -> Json<Value> { Json(json!({"sessions":[]})) }
pub async fn create(Json(body): Json<Value>) -> Json<Value> {
    let id = Uuid::new_v4();
    Json(json!({"id":id,"status":"active","task":body.get("task")}))
}
pub async fn get(Path(id): Path<String>) -> Json<Value> { Json(json!({"id":id,"status":"active"})) }
pub async fn delete(Path(id): Path<String>) -> Json<Value> { Json(json!({"deleted":id})) }
pub async fn stream(Path(id): Path<String>) -> Json<Value> { Json(json!({"stream":format!("sse://sessions/{}",id)})) }
pub async fn checkpoints(Path(id): Path<String>) -> Json<Value> { Json(json!({"checkpoints":[]})) }
pub async fn pause(Path(id): Path<String>) -> Json<Value> { Json(json!({"paused":id})) }
pub async fn resume(Path(id): Path<String>) -> Json<Value> { Json(json!({"resumed":id})) }
