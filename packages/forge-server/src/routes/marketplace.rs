use axum::extract::{Path, Query};
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Deserialize, Default)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub plugin_type: Option<String>,
}

pub async fn search(Query(q): Query<SearchQuery>) -> Json<Value> {
    let query = q.q.as_deref().unwrap_or("");
    Json(
        json!({"plugins":[],"query":query,"total":0,"message":"Marketplace registry not connected"}),
    )
}
pub async fn get_plugin(Path(name): Path<String>) -> Json<Value> {
    Json(json!({"name":name,"version":"0.1.0","description":"Plugin detail","error":"not_found"}))
}
pub async fn list_installed() -> Json<Value> {
    Json(json!({"installed":[],"total":0}))
}
pub async fn install(Json(body): Json<Value>) -> Json<Value> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
    Json(json!({"installed":name,"status":"ok"}))
}
pub async fn uninstall(Path(name): Path<String>) -> Json<Value> {
    Json(json!({"uninstalled":name,"status":"ok"}))
}
pub async fn publish(Json(body): Json<Value>) -> Json<Value> {
    let name = body.get("name").and_then(|v| v.as_str()).unwrap_or("");
    Json(json!({"published":name,"status":"queued"}))
}
