//! In-memory session store for the Forge server.
//!
//! Holds all active and completed sessions. Each session has a broadcast
//! channel that WebSocket/SSE consumers subscribe to for live events.
//! Each session also holds a PluginRegistry for running the harness pipeline
//! (observers → detectors → strategies) on ingested events.

use chrono::{DateTime, Utc};
use forge_sdk::events::{AgentEvent, Intervention};
use forge_sdk::harness::HarnessRunResult;
use forge_sdk::types::health::HealthScore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, watch, RwLock};

/// Status of a harness session.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Running => write!(f, "running"),
            Self::Paused => write!(f, "paused"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Per-session state stored in the session store.
pub struct SessionState {
    pub id: String,
    pub task: String,
    pub agent_type: String,
    pub preset: String,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub health_score: Option<HealthScore>,
    pub result: Option<HarnessRunResult>,
    /// Ring buffer of recent agent events (last 1000)
    pub events: Vec<AgentEvent>,
    /// Original hook names for each event (parallel to `events`)
    pub event_hooks: Vec<String>,
    /// Broadcast channel for live WebSocket/SSE consumers
    pub event_broadcaster: broadcast::Sender<AgentEvent>,
    /// Channel to send interventions INTO the running session
    pub intervention_tx: mpsc::Sender<Intervention>,
    /// Channel to cancel a running session
    pub cancel_tx: watch::Sender<bool>,
    // ── Harness pipeline state ──
    /// Accumulated observer results (dimension → latest value)
    pub observations: Vec<serde_json::Value>,
    /// Detected issues from detector runs (as JSON)
    pub detections: Vec<serde_json::Value>,
    /// Strategy results from intervention evaluation (as JSON)
    pub strategy_results: Vec<serde_json::Value>,
    /// Applied interventions (as JSON)
    pub interventions: Vec<serde_json::Value>,
    /// Count of events ingested (used for cycle scheduling)
    pub event_count: u64,
    // ── Cumulative analysis state ──
    /// Total input tokens across all turns
    pub total_input_tokens: u64,
    /// Total output tokens across all turns
    pub total_output_tokens: u64,
    /// Total cache read tokens
    pub total_cache_read: u64,
    /// Total cache write tokens
    pub total_cache_write: u64,
    /// Per-tool call counts
    pub tool_counts: HashMap<String, u64>,
    /// Per-tool error counts
    pub tool_errors: HashMap<String, u64>,
    /// Total tool call duration in ms
    pub total_tool_ms: u64,
    /// Context pressure readings over time
    pub context_pressure_history: Vec<f64>,
    /// Repeated tool call patterns detected (tool_name:repeat_count)
    pub loop_patterns: Vec<String>,
    /// Degradation warnings collected during session
    pub degradation_warnings: Vec<String>,
    /// Model name(s) detected
    pub model_name: Option<String>,
    /// Why the session stopped
    pub stop_reason: Option<String>,
    /// Number of subagents forked
    pub subagent_count: u64,
    /// User prompt count
    pub user_prompt_count: u64,
    /// History of user prompts captured during the session
    pub prompt_history: Vec<String>,
    /// Tools that have already had loop detections reported (prevents duplicates)
    pub loop_detected_tools: std::collections::HashSet<String>,
    /// Detection categories already reported (prevents duplicate categories)
    pub reported_categories: std::collections::HashSet<String>,
}

/// Thread-safe shared session store.
pub type SharedSessionStore = Arc<RwLock<HashMap<String, SessionState>>>;

/// Create a new empty session store.
pub fn new_store() -> SharedSessionStore {
    Arc::new(RwLock::new(HashMap::new()))
}

/// Persist all sessions to JSON files in ~/.forge/sessions/
pub async fn save_sessions(store: &SharedSessionStore) {
    let sessions = store.read().await;
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".forge")
        .join("sessions");
    let _ = std::fs::create_dir_all(&dir);

    for (id, s) in sessions.iter() {
        let path = dir.join(format!("{}.json", id));
        // Serialize key fields to JSON
        let data = serde_json::json!({
            "id": s.id,
            "task": s.task,
            "agent_type": s.agent_type,
            "preset": s.preset,
            "status": s.status,
            "created_at": s.created_at.to_rfc3339(),
            "completed_at": s.completed_at.map(|t| t.to_rfc3339()),
            "event_count": s.event_count,
            "total_input_tokens": s.total_input_tokens,
            "total_output_tokens": s.total_output_tokens,
            "total_cache_read": s.total_cache_read,
            "total_cache_write": s.total_cache_write,
            "tool_counts": s.tool_counts,
            "tool_errors": s.tool_errors,
            "context_pressure_history": s.context_pressure_history,
            "model_name": s.model_name,
            "stop_reason": s.stop_reason,
            "subagent_count": s.subagent_count,
            "user_prompt_count": s.user_prompt_count,
            "prompt_history": s.prompt_history,
            "observations_count": s.observations.len(),
            "detections_count": s.detections.len(),
            "interventions_count": s.interventions.len(),
            "health_score": s.health_score,
            // Persist last 200 events as JSON values (skip raw AgentEvent since
            // MessageContent doesn't serialize cleanly — cumulative stats suffice)
            "events": s.events.iter().rev().take(200).filter_map(|e| {
                serde_json::to_value(e).ok()
            }).collect::<Vec<_>>(),
        });
        let _ = std::fs::write(
            &path,
            serde_json::to_string_pretty(&data).unwrap_or_default(),
        );
    }
}

/// Load persisted sessions from ~/.forge/sessions/
pub async fn load_sessions(store: &SharedSessionStore) -> u64 {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".forge")
        .join("sessions");
    if !dir.exists() {
        return 0;
    }
    let mut count = 0u64;
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    let id = data["id"].as_str().unwrap_or("").to_string();
                    let task = data["task"].as_str().unwrap_or("").to_string();
                    let agent_type = data["agent_type"].as_str().unwrap_or("solo").to_string();
                    let preset = data["preset"].as_str().unwrap_or("solo").to_string();
                    let mut session = SessionState::new(id.clone(), task, agent_type, preset);
                    session.event_count = data["event_count"].as_u64().unwrap_or(0);
                    session.total_input_tokens = data["total_input_tokens"].as_u64().unwrap_or(0);
                    session.total_output_tokens = data["total_output_tokens"].as_u64().unwrap_or(0);
                    session.total_cache_read = data["total_cache_read"].as_u64().unwrap_or(0);
                    session.total_cache_write = data["total_cache_write"].as_u64().unwrap_or(0);
                    session.model_name = data["model_name"].as_str().map(String::from);
                    session.stop_reason = data["stop_reason"].as_str().map(String::from);
                    if let Some(events) = data["events"].as_array() {
                        for ev in events {
                            if let Ok(ae) = serde_json::from_value(ev.clone()) {
                                session.events.push(ae);
                            }
                        }
                        session.event_count = session.events.len() as u64;
                    }
                    session.subagent_count = data["subagent_count"].as_u64().unwrap_or(0);
                    session.user_prompt_count = data["user_prompt_count"].as_u64().unwrap_or(0);
                    if let Some(ph) = data["prompt_history"].as_array() {
                        session.prompt_history = ph
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                    if let Some(_hs) = &data["health_score"].as_object() {
                        session.health_score =
                            serde_json::from_value(data["health_score"].clone()).ok();
                    }
                    // Restore status
                    if let Some(status_str) = data["status"].as_str() {
                        session.status = match status_str {
                            "running" => SessionStatus::Running,
                            "completed" => SessionStatus::Completed,
                            "failed" => SessionStatus::Failed,
                            "paused" => SessionStatus::Paused,
                            _ => SessionStatus::Pending,
                        };
                    }
                    let mut sessions = store.write().await;
                    sessions.insert(id, session);
                    count += 1;
                }
            }
        }
    }
    count
}

// Add dirs dependency — use home_dir helper
mod dirs {
    pub fn home_dir() -> Option<std::path::PathBuf> {
        std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .map(std::path::PathBuf::from)
            .ok()
    }
}

impl SessionState {
    /// Create a new session state with fresh channels.
    pub fn new(id: String, task: String, agent_type: String, preset: String) -> Self {
        let (event_broadcaster, _) = broadcast::channel(256);
        let (intervention_tx, _intervention_rx) = mpsc::channel(64);
        let (cancel_tx, _cancel_rx) = watch::channel(false);
        Self {
            id,
            task,
            agent_type,
            preset,
            status: SessionStatus::Pending,
            created_at: Utc::now(),
            completed_at: None,
            health_score: None,
            result: None,
            events: Vec::new(),
            event_hooks: Vec::new(),
            event_broadcaster,
            intervention_tx,
            cancel_tx,
            observations: Vec::new(),
            detections: Vec::new(),
            strategy_results: Vec::new(),
            interventions: Vec::new(),
            event_count: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cache_read: 0,
            total_cache_write: 0,
            tool_counts: HashMap::new(),
            tool_errors: HashMap::new(),
            total_tool_ms: 0,
            context_pressure_history: Vec::new(),
            loop_patterns: Vec::new(),
            degradation_warnings: Vec::new(),
            model_name: None,
            stop_reason: None,
            subagent_count: 0,
            user_prompt_count: 0,
            prompt_history: Vec::new(),
            loop_detected_tools: std::collections::HashSet::new(),
            reported_categories: std::collections::HashSet::new(),
        }
    }
}
