use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct ComplianceGapDetector;

#[async_trait]
impl Detector for ComplianceGapDetector {
    fn name(&self) -> &'static str { "compliance_gap" }
    fn description(&self) -> &'static str { "Detects skipped human gates, audit gaps, PII exposure" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let mut issues = Vec::new();
        for o in obs {
            if let Some(gap) = o.get("compliance_gap").and_then(|v| v.as_str()) {
                issues.push(DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                    severity: Severity::Critical,
                    category: IssueCategory::ComplianceGap { gap_type: gap.into() },
                    description: format!("Compliance gap: {}", gap),
                    confidence: 1.0,
                    suggested_actions: vec!["circuit_break".into(), "pause".into(), "quarantine".into()],
                    evidence_summary: format!("Gap detected: {}", gap) });
            }
        }
        issues
    }
}
