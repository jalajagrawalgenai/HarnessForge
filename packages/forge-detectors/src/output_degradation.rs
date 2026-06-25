use async_trait::async_trait;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use uuid::Uuid;

pub struct OutputDegradationDetector {
    decline_threshold: f64,
}
impl OutputDegradationDetector {
    pub fn new(t: f64) -> Self {
        Self {
            decline_threshold: t,
        }
    }
}

#[async_trait]
impl Detector for OutputDegradationDetector {
    fn name(&self) -> &'static str {
        "output_degradation"
    }
    fn description(&self) -> &'static str {
        "Detects declining output quality over time"
    }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let scores: Vec<f64> = obs
            .iter()
            .filter_map(|o| o.get("quality_score").and_then(|v| v.as_f64()))
            .collect();
        if scores.len() < 4 {
            return vec![];
        }
        let recent = &scores[scores.len() - 4..];
        let trend: f64 = recent.windows(2).map(|w| w[1] - w[0]).sum();
        let avg_decline = -trend / 3.0;
        if avg_decline > self.decline_threshold {
            vec![DetectedIssue {
                id: Uuid::new_v4(),
                agent_id: agent_id.into(),
                severity: if avg_decline > self.decline_threshold * 2.0 {
                    Severity::Error
                } else {
                    Severity::Warning
                },
                category: IssueCategory::OutputDegradation {
                    trend_slope: -avg_decline,
                    consecutive_declines: 3,
                },
                description: format!("Output quality declining ({:.2}/turn)", avg_decline),
                confidence: (avg_decline / self.decline_threshold).min(1.0),
                suggested_actions: vec!["replace".into(), "compact".into()],
                evidence_summary: format!("Quality decline: {:.2}/turn over 3 turns", avg_decline),
            }]
        } else {
            vec![]
        }
    }
}
