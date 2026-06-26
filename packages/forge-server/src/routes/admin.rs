use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;

static API_KEYS: std::sync::LazyLock<Mutex<HashMap<String, Value>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn list_keys() -> Json<Value> {
    let keys = API_KEYS.lock().unwrap();
    let items: Vec<Value> = keys.values().cloned().collect();
    Json(json!({"keys": items, "total": items.len()}))
}

pub async fn create_key(Json(body): Json<Value>) -> Json<Value> {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let scopes: Vec<String> = body
        .get("scopes")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let key = format!("fk_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
    let key_data = json!({
        "name": name, "key": key, "scopes": scopes,
        "created_at": chrono::Utc::now().to_rfc3339()
    });
    API_KEYS
        .lock()
        .unwrap()
        .insert(key.clone(), key_data.clone());
    Json(json!({"key": key_data, "created": true}))
}

pub async fn revoke_key(Path(id): Path<String>) -> Json<Value> {
    API_KEYS.lock().unwrap().remove(&id);
    Json(json!({"revoked": id}))
}

pub async fn get_quotas() -> Json<Value> {
    Json(json!({"quotas": [
        {"type": "sessions_per_day", "limit": 100, "current": 0, "is_hard_limit": false},
        {"type": "tokens_per_month", "limit": 10000000, "current": 0, "is_hard_limit": true},
        {"type": "cost_per_month", "limit": 500.0, "current": 0.0, "is_hard_limit": true}
    ]}))
}

pub async fn update_quotas(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"updated": true, "quotas": body}))
}
