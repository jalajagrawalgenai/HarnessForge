use axum::body::Body;
use forge_server::create_app;
use forge_server::session::store::new_store;
use http_body_util::BodyExt;
use tower::ServiceExt;

fn app() -> axum::Router {
    create_app(new_store())
}

async fn get(path: &str) -> (u16, serde_json::Value) {
    let req = http::Request::builder()
        .uri(path)
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, body)
}

async fn post(path: &str, data: &serde_json::Value) -> (u16, serde_json::Value) {
    let req = http::Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_vec(data).unwrap()))
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    let status = resp.status().as_u16();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap_or_default();
    (status, body)
}

#[tokio::test]
async fn health_ok() {
    let (s, b) = get("/api/v1/health").await;
    assert_eq!(s, 200);
    assert_eq!(b["status"], "ok");
}
#[tokio::test]
async fn status_has_version() {
    let (s, b) = get("/api/v1/status").await;
    assert_eq!(s, 200);
    assert!(b["version"].as_str().is_some());
}
#[tokio::test]
async fn readiness_ok() {
    let (s, b) = get("/api/v1/health/readiness").await;
    assert_eq!(s, 200);
    assert_eq!(b["ready"], true);
}
#[tokio::test]
async fn harness_has_counts() {
    let (s, b) = get("/api/v1/harness").await;
    assert_eq!(s, 200);
    assert_eq!(b["observers"]["count"], 12);
    assert_eq!(b["detectors"]["count"], 16);
    assert_eq!(b["strategies"]["count"], 14);
}
#[tokio::test]
async fn sessions_create() {
    // Manual creation now returns a helpful message
    let (s, b) = post(
        "/api/v1/sessions",
        &serde_json::json!({"task":"test","agent_type":"solo","preset":"solo"}),
    )
    .await;
    assert_eq!(s, 200);
    assert!(b["error"].as_str().is_some());
    assert_eq!(b["how"], "Just use Claude Code, LangGraph, CrewAI, or any agent normally.");
}
#[tokio::test]
async fn sessions_list() {
    let app = forge_server::create_app(new_store());
    // Create session via ingest API (real agent path)
    let req1 = http::Request::builder()
        .method("POST")
        .uri("/api/v1/ingest/event")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"agentClass":"claude-code","sessionId":"test-list-1","agentId":"ag1","hookName":"SessionStart","payload":{"prompt":"test task"},"flags":{"startsSession":true}}"#,
        ))
        .unwrap();
    let _ = app.clone().oneshot(req1).await.unwrap();
    let req2 = http::Request::builder()
        .uri("/api/v1/sessions")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req2).await.unwrap();
    let bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["total"].as_u64().unwrap(), 1);
}
#[tokio::test]
async fn session_not_found() {
    let (s, b) = get("/api/v1/sessions/nope").await;
    assert_eq!(s, 200);
    assert!(b["error"].as_str().is_some());
}
#[tokio::test]
async fn compliance_frameworks() {
    let (s, b) = get("/api/v1/compliance/frameworks").await;
    assert_eq!(s, 200);
    let fw = b["frameworks"].as_array().unwrap();
    assert!(fw.iter().any(|f| f["id"] == "EuAiAct"));
}
#[tokio::test]
async fn compliance_report() {
    let (s, b) = get("/api/v1/compliance/report?framework=EuAiAct&session_id=x").await;
    assert_eq!(s, 200);
    assert!(b["checks"].as_array().is_some());
}
#[tokio::test]
async fn skills_list() {
    let (s, b) = get("/api/v1/skills").await;
    assert_eq!(s, 200);
    assert!(b["total"].as_u64().unwrap() >= 3);
}
#[tokio::test]
async fn mcp_servers() {
    let (s, b) = get("/api/v1/mcp/servers").await;
    assert_eq!(s, 200);
    assert!(b["count"].as_u64().is_some());
}
#[tokio::test]
async fn mcp_add() {
    let (s, b) = post(
        "/api/v1/mcp/servers",
        &serde_json::json!({"name":"s","transport":"stdio","endpoint":"e"}),
    )
    .await;
    assert_eq!(s, 200);
    assert_eq!(b["added"], "s");
}
#[tokio::test]
async fn auth_config() {
    let (s, b) = get("/api/v1/auth/config").await;
    assert_eq!(s, 200);
    assert!(b["providers"].as_array().is_some());
}
#[tokio::test]
async fn export_configs() {
    let (s, b) = get("/api/v1/export/configs").await;
    assert_eq!(s, 200);
    assert!(b["configs"].as_array().unwrap().len() >= 5);
}
#[tokio::test]
async fn cloud_providers() {
    let (s, b) = get("/api/v1/cloud/providers").await;
    assert_eq!(s, 200);
    let p = b["providers"].as_array().unwrap();
    assert!(p.iter().any(|x| x["name"] == "aws"));
}
#[tokio::test]
async fn analytics_overview() {
    let (s, b) = get("/api/v1/analytics/overview").await;
    assert_eq!(s, 200);
    assert!(b["total_sessions"].as_u64().is_some());
}
#[tokio::test]
async fn admin_keys() {
    let (s, b) = get("/api/v1/admin/keys").await;
    assert_eq!(s, 200);
    assert!(b["keys"].as_array().is_some());
}
#[tokio::test]
async fn admin_create_key() {
    let (s, b) = post(
        "/api/v1/admin/keys",
        &serde_json::json!({"name":"k","scopes":["r"]}),
    )
    .await;
    assert_eq!(s, 200);
    assert_eq!(b["created"], true);
}
#[tokio::test]
async fn meta_not_enough_data() {
    let (s, b) = post("/api/v1/meta/improve", &serde_json::json!({})).await;
    assert_eq!(s, 200);
    assert_eq!(b["status"], "not_enough_data");
}
#[tokio::test]
async fn dashboard_served() {
    let req = http::Request::builder()
        .uri("/")
        .body(Body::empty())
        .unwrap();
    let resp = app().oneshot(req).await.unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}
