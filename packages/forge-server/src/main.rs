use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;

mod db;
mod middleware;
mod routes;
mod ws;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/v1/health", get(health))
        .route("/v1/status", get(status))
        .route(
            "/v1/sessions",
            get(routes::sessions::list).post(routes::sessions::create),
        )
        .route(
            "/v1/sessions/:id",
            get(routes::sessions::get).delete(routes::sessions::delete),
        )
        .route("/v1/sessions/:id/stream", get(routes::sessions::stream))
        .route("/v1/sessions/:id/audit", get(routes::audit::get_audit))
        .route(
            "/v1/sessions/:id/audit/report",
            get(routes::audit::get_report),
        )
        .route(
            "/v1/sessions/:id/audit/export",
            get(routes::audit::export_audit),
        )
        .route("/v1/sessions/:id/audit/replay", get(routes::audit::replay))
        .route(
            "/v1/sessions/:id/checkpoints",
            get(routes::sessions::checkpoints),
        )
        .route(
            "/v1/sessions/:id/pause",
            axum::routing::post(routes::sessions::pause),
        )
        .route(
            "/v1/sessions/:id/resume",
            axum::routing::post(routes::sessions::resume),
        )
        .route(
            "/v1/harness",
            get(routes::harness::get).put(routes::harness::update),
        )
        .route("/v1/harness/versions", get(routes::harness::versions))
        .route(
            "/v1/meta/improve",
            axum::routing::post(routes::meta::improve),
        )
        .route("/v1/meta/weaknesses", get(routes::meta::weaknesses))
        .route("/v1/meta/edits", get(routes::meta::edits))
        .route("/v1/analytics/overview", get(routes::analytics::overview))
        .route("/v1/analytics/tokens", get(routes::analytics::tokens))
        .route("/v1/analytics/costs", get(routes::analytics::costs))
        .route(
            "/v1/analytics/interventions",
            get(routes::analytics::interventions),
        )
        .route("/v1/analytics/health", get(routes::analytics::health))
        .route(
            "/v1/admin/keys",
            get(routes::admin::list_keys).post(routes::admin::create_key),
        )
        .route("/v1/admin/quotas", get(routes::admin::quotas));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Forge server starting on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> Json<Value> {
    Json(json!({"status":"ok"}))
}
async fn status() -> Json<Value> {
    Json(json!({"version":"0.1.0","uptime":0}))
}
