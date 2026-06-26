use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

pub async fn get_config() -> Json<Value> {
    Json(json!({
        "sso": null,
        "jit_provisioning": {"enabled":false,"default_role":"viewer","allowed_domains":null},
        "session_duration_hours": 24,
        "mfa_required": false,
        "providers": ["okta","azure_ad","google_workspace","custom_oidc"]
    }))
}
pub async fn update_config(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"updated":true,"config":body}))
}
pub async fn status() -> Json<Value> {
    Json(json!({"authenticated":false,"user":null,"message":"SSO not configured. Running in local mode."}))
}
pub async fn list_users() -> Json<Value> {
    Json(json!({"users":[],"total":0}))
}
pub async fn invite_user(Json(body): Json<Value>) -> Json<Value> {
    let email = body.get("email").and_then(|v|v.as_str()).unwrap_or("");
    Json(json!({"invited":email}))
}
pub async fn remove_user(Path(email): Path<String>) -> Json<Value> {
    Json(json!({"removed":email}))
}
pub async fn test_connection() -> Json<Value> {
    Json(json!({"connected":false,"message":"SSO not configured"}))
}
