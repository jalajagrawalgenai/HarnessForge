pub mod db;
pub mod middleware;
pub mod routes;
pub mod session;
pub mod ws;

use crate::session::store::SharedSessionStore;
use axum::body::Body;
use axum::http::{header, Response, StatusCode};
use axum::response::IntoResponse;
use axum::Router;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    pub store: SharedSessionStore,
}

pub fn create_app(store: SharedSessionStore) -> Router {
    let state = Arc::new(AppState { store });

    Router::new()
        .route("/api/v1/health", axum::routing::get(routes::health::get))
        .route("/api/v1/status", axum::routing::get(routes::health::status))
        .route(
            "/api/v1/health/readiness",
            axum::routing::get(routes::health::readiness),
        )
        .route(
            "/api/v1/sessions",
            axum::routing::get(routes::sessions::list).post(routes::sessions::create),
        )
        .route(
            "/api/v1/sessions/{id}",
            axum::routing::get(routes::sessions::get).delete(routes::sessions::delete),
        )
        .route(
            "/api/v1/sessions/{id}/stream",
            axum::routing::get(routes::stream::session_stream),
        )
        .route(
            "/api/v1/sessions/{id}/checkpoints",
            axum::routing::get(routes::sessions::checkpoints),
        )
        .route(
            "/api/v1/sessions/{id}/pause",
            axum::routing::post(routes::sessions::pause),
        )
        .route(
            "/api/v1/sessions/{id}/resume",
            axum::routing::post(routes::sessions::resume),
        )
        .route(
            "/api/v1/sessions/{id}/audit",
            axum::routing::get(routes::audit::get_audit),
        )
        .route(
            "/api/v1/sessions/{id}/audit/report",
            axum::routing::get(routes::audit::get_report),
        )
        .route(
            "/api/v1/sessions/{id}/audit/export",
            axum::routing::get(routes::audit::export_audit),
        )
        .route(
            "/api/v1/sessions/{id}/audit/replay",
            axum::routing::get(routes::audit::replay),
        )
        .route(
            "/api/v1/harness",
            axum::routing::get(routes::harness::get).put(routes::harness::update),
        )
        .route(
            "/api/v1/harness/versions",
            axum::routing::get(routes::harness::versions),
        )
        .route(
            "/api/v1/harness/detectors/efficacy",
            axum::routing::get(routes::harness::detector_efficacy),
        )
        .route(
            "/api/v1/harness/strategies/efficacy",
            axum::routing::get(routes::harness::strategy_efficacy),
        )
        .route(
            "/api/v1/compliance/frameworks",
            axum::routing::get(routes::compliance::list_frameworks),
        )
        .route(
            "/api/v1/compliance/report",
            axum::routing::get(routes::compliance::get_report)
                .post(routes::compliance::generate_report),
        )
        .route(
            "/api/v1/compliance/checklist",
            axum::routing::get(routes::compliance::get_checklist),
        )
        .route("/api/v1/skills", axum::routing::get(routes::skills::list))
        .route(
            "/api/v1/skills/{name}",
            axum::routing::get(routes::skills::get),
        )
        .route(
            "/api/v1/skills/compose",
            axum::routing::post(routes::skills::compose),
        )
        .route(
            "/api/v1/skills/validate",
            axum::routing::post(routes::skills::validate),
        )
        .route(
            "/api/v1/skills/scaffold",
            axum::routing::post(routes::skills::scaffold),
        )
        .route(
            "/api/v1/mcp/servers",
            axum::routing::get(routes::mcp_routes::list_servers)
                .post(routes::mcp_routes::add_server),
        )
        .route(
            "/api/v1/mcp/servers/{name}",
            axum::routing::delete(routes::mcp_routes::remove_server),
        )
        .route(
            "/api/v1/mcp/discover",
            axum::routing::get(routes::mcp_routes::discover),
        )
        .route(
            "/api/v1/mcp/gateway",
            axum::routing::get(routes::mcp_routes::gateway_status)
                .post(routes::mcp_routes::start_gateway),
        )
        .route(
            "/api/v1/mcp/server",
            axum::routing::get(routes::mcp_routes::forge_server_status)
                .post(routes::mcp_routes::start_forge_server),
        )
        .route(
            "/api/v1/mcp/server/tools",
            axum::routing::get(routes::mcp_routes::list_tools),
        )
        .route(
            "/api/v1/mcp/server/resources",
            axum::routing::get(routes::mcp_routes::list_resources),
        )
        .route(
            "/api/v1/auth/config",
            axum::routing::get(routes::auth::get_config).put(routes::auth::update_config),
        )
        .route(
            "/api/v1/auth/status",
            axum::routing::get(routes::auth::status),
        )
        .route(
            "/api/v1/auth/users",
            axum::routing::get(routes::auth::list_users).post(routes::auth::invite_user),
        )
        .route(
            "/api/v1/auth/users/{email}",
            axum::routing::delete(routes::auth::remove_user),
        )
        .route(
            "/api/v1/auth/test",
            axum::routing::post(routes::auth::test_connection),
        )
        .route(
            "/api/v1/export/configs",
            axum::routing::get(routes::export_routes::list_configs)
                .put(routes::export_routes::update_config),
        )
        .route(
            "/api/v1/export/test/{target}",
            axum::routing::post(routes::export_routes::test_connection),
        )
        .route(
            "/api/v1/export/trigger",
            axum::routing::post(routes::export_routes::trigger_export),
        )
        .route(
            "/api/v1/export/alerting",
            axum::routing::get(routes::export_routes::get_alert_config)
                .put(routes::export_routes::update_alert_config),
        )
        .route(
            "/api/v1/marketplace/search",
            axum::routing::get(routes::marketplace::search),
        )
        .route(
            "/api/v1/marketplace/plugins/{name}",
            axum::routing::get(routes::marketplace::get_plugin)
                .delete(routes::marketplace::uninstall),
        )
        .route(
            "/api/v1/marketplace/installed",
            axum::routing::get(routes::marketplace::list_installed),
        )
        .route(
            "/api/v1/marketplace/install",
            axum::routing::post(routes::marketplace::install),
        )
        .route(
            "/api/v1/marketplace/publish",
            axum::routing::post(routes::marketplace::publish),
        )
        .route(
            "/api/v1/cloud/providers",
            axum::routing::get(routes::cloud::list_providers),
        )
        .route(
            "/api/v1/cloud/providers/{provider}/configure",
            axum::routing::post(routes::cloud::configure),
        )
        .route(
            "/api/v1/cloud/providers/{provider}/status",
            axum::routing::get(routes::cloud::provider_status),
        )
        .route(
            "/api/v1/cloud/deploy",
            axum::routing::post(routes::cloud::deploy),
        )
        .route(
            "/api/v1/analytics/overview",
            axum::routing::get(routes::analytics::overview),
        )
        .route(
            "/api/v1/analytics/tokens",
            axum::routing::get(routes::analytics::tokens),
        )
        .route(
            "/api/v1/analytics/costs",
            axum::routing::get(routes::analytics::costs),
        )
        .route(
            "/api/v1/analytics/interventions",
            axum::routing::get(routes::analytics::interventions),
        )
        .route(
            "/api/v1/analytics/health",
            axum::routing::get(routes::analytics::health),
        )
        .route(
            "/api/v1/meta/improve",
            axum::routing::post(routes::meta::improve),
        )
        .route(
            "/api/v1/meta/weaknesses",
            axum::routing::get(routes::meta::weaknesses),
        )
        .route(
            "/api/v1/meta/edits",
            axum::routing::get(routes::meta::edits),
        )
        .route(
            "/api/v1/meta/edits/{id}/accept",
            axum::routing::post(routes::meta::accept_edit),
        )
        .route(
            "/api/v1/meta/edits/{id}/reject",
            axum::routing::post(routes::meta::reject_edit),
        )
        .route(
            "/api/v1/meta/ab-tests",
            axum::routing::get(routes::meta::ab_tests),
        )
        .route(
            "/api/v1/admin/keys",
            axum::routing::get(routes::admin::list_keys).post(routes::admin::create_key),
        )
        .route(
            "/api/v1/admin/keys/{id}",
            axum::routing::delete(routes::admin::revoke_key),
        )
        .route(
            "/api/v1/admin/quotas",
            axum::routing::get(routes::admin::get_quotas).put(routes::admin::update_quotas),
        )
        .route("/ws", axum::routing::get(ws::handler::ws_handler))
        .route("/style.css", axum::routing::get(serve_css))
        .route("/app.js", axum::routing::get(serve_js))
        .route("/", axum::routing::get(serve_index))
        .fallback(serve_index)
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// Embedded static files — compiled into the binary, always available
const INDEX_HTML: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/index.html"));
const STYLE_CSS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/style.css"));
const APP_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/static/app.js"));

async fn serve_index() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(INDEX_HTML))
        .unwrap()
}

async fn serve_css() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/css; charset=utf-8")
        .body(Body::from(STYLE_CSS))
        .unwrap()
}

async fn serve_js() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            "application/javascript; charset=utf-8",
        )
        .body(Body::from(APP_JS))
        .unwrap()
}

pub async fn run_server(start_port: u16) {
    let store = session::store::new_store();
    let app = create_app(store);

    // Try ports starting from start_port, find the first available one
    let mut port = start_port;
    let listener = loop {
        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => break listener,
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
                tracing::warn!("Port {} is in use, trying port {}...", port, port + 1);
                port += 1;
                if port > start_port + 100 {
                    panic!("No free port found between {} and {}", start_port, port);
                }
            }
            Err(e) => panic!("Failed to bind to port {}: {}", port, e),
        }
    };

    let addr = listener.local_addr().unwrap();
    tracing::info!("Forge server started on http://{}", addr);
    tracing::info!("Dashboard: http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
