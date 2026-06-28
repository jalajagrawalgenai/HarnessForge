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
}

/// Thread-safe shared session store.
pub type SharedSessionStore = Arc<RwLock<HashMap<String, SessionState>>>;

/// Create a new empty session store.
pub fn new_store() -> SharedSessionStore {
    Arc::new(RwLock::new(HashMap::new()))
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
        }
    }
}
