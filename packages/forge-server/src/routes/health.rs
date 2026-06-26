//! Health check and status endpoints.

use axum::Json;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

static START_TIME: std::sync::LazyLock<Instant> = std::sync::LazyLock::new(Instant::now);
static REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);

pub async fn get() -> Json<Value> {
    REQUEST_COUNT.fetch_add(1, Ordering::Relaxed);
    Json(json!({
        "status": "ok",
        "uptime_secs": START_TIME.elapsed().as_secs(),
        "requests": REQUEST_COUNT.load(Ordering::Relaxed),
    }))
}

pub async fn status() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "uptime_secs": START_TIME.elapsed().as_secs(),
        "requests": REQUEST_COUNT.load(Ordering::Relaxed),
    }))
}

pub async fn readiness() -> Json<Value> {
    Json(json!({
        "ready": true,
        "store": "in_memory",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
