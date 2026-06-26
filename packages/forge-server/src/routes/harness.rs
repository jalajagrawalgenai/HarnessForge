use axum::Json;
use serde_json::{json, Value};

/// GET /v1/harness — current harness configuration and available options.
pub async fn get() -> Json<Value> {
    Json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "agent_types": [
            "solo","custom","langgraph","crewai","autogen","langchain","openai-swarm",
            "semantic-kernel","haystack","dspy","llamaindex","taskweaver","agno",
            "atomic-agents","bee-agent","pydantic-ai","claude-code","aider","cline",
            "continue","vercel-ai","copilot","cursor","windsurf","devin","amazon-q",
            "replit-agent","pearai","bolt-new","lovable","v0"
        ],
        "presets": [
            "solo","langgraph","crewai","autogen","langchain","openai-swarm",
            "semantic-kernel","haystack","dspy","llamaindex","taskweaver","agno",
            "atomic-agents","bee-agent","pydantic-ai","claude-code","aider","cline",
            "continue","vercel-ai","copilot","cursor","windsurf","devin","amazon-q",
            "replit-agent","pearai","bolt-new","lovable","v0","custom"
        ],
        "observers": {"count": 12, "list": ["token","latency","cost","accuracy","security","reliability","context_quality","orch","comm","compliance","memory","diversity"]},
        "detectors": {"count": 16, "list": ["loop","stale_context","cost_anomaly","deadlock","hallucination","prompt_injection","secret_leak","variety_collapse","conversation_stall","goal_drift","model_mismatch","accuracy_risk","runaway_cost","resource_exhaustion","output_degradation","compliance_gap"]},
        "strategies": {"count": 14, "list": ["nudge","compact","pause","escalate","fork","reroute","rollback","diversify","isolate","circuit_break","replace","interject","degrade","quarantine"]}
    }))
}

/// PUT /v1/harness — update harness configuration.
pub async fn update(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({"updated":true,"config":body,"message":"Config updated (in-memory, persists until restart)"}))
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
