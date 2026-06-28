use crate::AppState;
use axum::extract::State;
use axum::Json;
use serde_json::{json, Value};
use std::sync::Arc;

/// GET /v1/harness — current harness configuration with enabled state.
pub async fn get(State(state): State<Arc<AppState>>) -> Json<Value> {
    let cfg = state.harness_config.read().await;
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "preset": cfg.preset,
        "dry_run": cfg.dry_run,
        "observers": cfg.enabled_observers,
        "detectors": cfg.enabled_detectors,
        "strategies": cfg.enabled_strategies,
        "all_observers": ["token","latency","cost","accuracy","security","reliability","context_quality","orch","comm","compliance","memory","diversity"],
        "all_detectors": ["loop","stale_context","cost_anomaly","deadlock","hallucination","prompt_injection","secret_leak","variety_collapse","conversation_stall","goal_drift","model_mismatch","accuracy_risk","runaway_cost","resource_exhaustion","output_degradation","compliance_gap"],
        "all_strategies": ["nudge","compact","pause","escalate","fork","reroute","rollback","diversify","isolate","circuit_break","replace","interject","degrade","quarantine"],
    }))
}

/// PUT /v1/harness — update which observers/detectors/strategies are enabled.
/// Body: { "type": "observer|detector|strategy", "id": "token", "enabled": true }
pub async fn update(State(state): State<Arc<AppState>>, Json(body): Json<Value>) -> Json<Value> {
    let typ = body.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let id = body.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let enabled = body
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut cfg = state.harness_config.write().await;

    match typ {
        "observer" => {
            if enabled && !cfg.enabled_observers.contains(&id.to_string()) {
                cfg.enabled_observers.push(id.to_string());
            } else if !enabled {
                cfg.enabled_observers.retain(|x| x != id);
            }
        }
        "detector" => {
            if enabled && !cfg.enabled_detectors.contains(&id.to_string()) {
                cfg.enabled_detectors.push(id.to_string());
            } else if !enabled {
                cfg.enabled_detectors.retain(|x| x != id);
            }
        }
        "strategy" => {
            if enabled && !cfg.enabled_strategies.contains(&id.to_string()) {
                cfg.enabled_strategies.push(id.to_string());
            } else if !enabled {
                cfg.enabled_strategies.retain(|x| x != id);
            }
        }
        "intervention" => {
            cfg.dry_run = !enabled;
        }
        _ => {}
    }

    Json(json!({
        "updated": true,
        "type": typ,
        "id": id,
        "enabled": enabled,
        "observers": cfg.enabled_observers,
        "detectors": cfg.enabled_detectors,
        "strategies": cfg.enabled_strategies,
        "dry_run": cfg.dry_run,
    }))
}

/// GET /v1/harness/versions
pub async fn versions() -> Json<Value> {
    Json(json!({"versions":[env!("CARGO_PKG_VERSION")],"current":env!("CARGO_PKG_VERSION")}))
}

/// GET /v1/harness/detectors/efficacy
pub async fn detector_efficacy() -> Json<Value> {
    Json(json!({"detectors":[],"message":"Efficacy data requires completed sessions"}))
}

/// GET /v1/harness/strategies/efficacy
pub async fn strategy_efficacy() -> Json<Value> {
    Json(json!({"strategies":[],"message":"Efficacy data requires completed sessions"}))
}
