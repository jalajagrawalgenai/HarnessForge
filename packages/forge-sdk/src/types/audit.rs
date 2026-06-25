// forge-sdk/src/types/audit.rs — Audit trail types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: i64,
    pub session_id: Uuid,
    pub agent_id: Option<Uuid>,
    pub trace_id: Uuid,
    pub sequence: i64,
    pub phase: AuditPhase,
    pub event_type: String,
    pub event_data: serde_json::Value,
    pub parent_event: Option<i64>,
    pub checkpoint_ref: Option<Uuid>,
    pub hash_chain: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditPhase {
    Observe,
    Detect,
    Strategy,
    Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: Uuid,
    pub event_id: i64,
    pub session_id: Uuid,
    pub agent_states: serde_json::Value,
    pub context_snapshot: Option<serde_json::Value>,
    pub message_queue: Option<serde_json::Value>,
    pub state_store: Option<serde_json::Value>,
    pub graph_state: Option<serde_json::Value>,
    pub task_progress: Option<serde_json::Value>,
    pub plan: Option<serde_json::Value>,
    pub token_usage: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub session_id: Uuid,
    pub task: String,
    pub agent_type: String,
    pub model: String,
    pub duration_secs: f64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub health_score: Option<f64>,
    pub observations: Vec<ObservationSummary>,
    pub detections: Vec<DetectionSummary>,
    pub interventions: Vec<InterventionSummary>,
    pub checkpoints: Vec<CheckpointSummary>,
    pub harness_effectiveness: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationSummary {
    pub dimension: String,
    pub event_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionSummary {
    pub turn: u32,
    pub detector: String,
    pub category: String,
    pub severity: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionSummary {
    pub turn: u32,
    pub strategy: String,
    pub outcome: String,
    pub impact: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointSummary {
    pub turn: u32,
    pub reason: String,
}
