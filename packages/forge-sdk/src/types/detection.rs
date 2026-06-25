// forge-sdk/src/types/detection.rs — Issue detection types

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedIssue {
    pub id: Uuid,
    pub agent_id: String,
    pub severity: Severity,
    pub category: IssueCategory,
    pub description: String,
    pub confidence: f64,
    pub suggested_actions: Vec<String>,
    pub evidence_summary: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueCategory {
    LoopDetected {
        tool_name: String,
        call_count: u32,
        no_progress_turns: u32,
    },
    StaleContext {
        file_path: String,
        read_count: u32,
        context_pressure: f64,
    },
    CostAnomaly {
        expected_cost: f64,
        actual_cost: f64,
        multiplier: f64,
    },
    Deadlock {
        agents: Vec<String>,
        wait_duration_secs: u64,
    },
    Hallucination {
        reference: String,
        reference_type: String,
    },
    PromptInjection {
        pattern_matched: String,
    },
    SecretLeak {
        secret_type: String,
    },
    VarietyCollapse {
        similarity_score: f64,
        agent_count: u32,
    },
    ConversationStall {
        duration_secs: u64,
    },
    GoalDrift {
        similarity_to_original: f64,
    },
    ModelMismatch {
        task_complexity: String,
        model_used: String,
        suggested_model: String,
    },
    AccuracyRisk {
        risk_factors: Vec<String>,
    },
    RunawayCost {
        acceleration: f64,
    },
    ResourceExhaustion {
        resource: String,
        usage_pct: f64,
    },
    OutputDegradation {
        trend_slope: f64,
        consecutive_declines: u32,
    },
    ComplianceGap {
        gap_type: String,
    },
}
