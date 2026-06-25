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

impl DetectedIssue {
    pub fn category_name(&self) -> &str {
        match &self.category {
            IssueCategory::LoopDetected { .. } => "loop",
            IssueCategory::StaleContext { .. } => "stale_context",
            IssueCategory::CostAnomaly { .. } => "cost_anomaly",
            IssueCategory::Deadlock { .. } => "deadlock",
            IssueCategory::Hallucination { .. } => "hallucination",
            IssueCategory::PromptInjection { .. } => "prompt_injection",
            IssueCategory::SecretLeak { .. } => "secret_leak",
            IssueCategory::VarietyCollapse { .. } => "variety_collapse",
            IssueCategory::ConversationStall { .. } => "conversation_stall",
            IssueCategory::GoalDrift { .. } => "goal_drift",
            IssueCategory::ModelMismatch { .. } => "model_mismatch",
            IssueCategory::AccuracyRisk { .. } => "accuracy_risk",
            IssueCategory::RunawayCost { .. } => "runaway_cost",
            IssueCategory::ResourceExhaustion { .. } => "resource_exhaustion",
            IssueCategory::OutputDegradation { .. } => "output_degradation",
            IssueCategory::ComplianceGap { .. } => "compliance_gap",
        }
    }
}

impl PartialOrd for Severity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Severity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let v = |s: &Severity| match s { Severity::Info=>0, Severity::Warning=>1, Severity::Error=>2, Severity::Critical=>3 };
        v(self).cmp(&v(other))
    }
}
