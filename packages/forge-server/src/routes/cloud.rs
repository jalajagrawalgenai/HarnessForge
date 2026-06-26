use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

pub async fn list_providers() -> Json<Value> {
    Json(json!({"providers":[
        {"name":"aws","regions":["us-east-1","us-west-2","eu-west-1","ap-southeast-1"],"status":"not_configured"},
        {"name":"azure","regions":["eastus","westeurope","southeastasia"],"status":"not_configured"},
        {"name":"gcp","regions":["us-central1","europe-west1","asia-east1"],"status":"not_configured"}
    ]}))
}
pub async fn configure(Path(provider): Path<String>, Json(body): Json<Value>) -> Json<Value> {
    Json(
        json!({"configured":provider,"region":body.get("region").and_then(|v|v.as_str()),"status":"ok"}),
    )
}
pub async fn provider_status(Path(provider): Path<String>) -> Json<Value> {
    Json(json!({"provider":provider,"status":"configured","connected":false}))
}
pub async fn deploy(Json(body): Json<Value>) -> Json<Value> {
    let provider = body
        .get("provider")
        .and_then(|v| v.as_str())
        .unwrap_or("aws");
    Json(json!({"deployed":true,"provider":provider,"message":"Deployment initiated"}))
}
