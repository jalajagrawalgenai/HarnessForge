use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct GoalDriftDetector { similarity_threshold: f64 }
impl GoalDriftDetector { pub fn new(t: f64) -> Self { Self { similarity_threshold: t } } }

#[async_trait]
impl Detector for GoalDriftDetector {
    fn name(&self) -> &'static str { "goal_drift" }
    fn description(&self) -> &'static str { "Detects when agent diverges from original task" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let original = obs.first().and_then(|o| o.get("original_task")).and_then(|v| v.as_str());
        let current = obs.last().and_then(|o| o.get("current_output")).and_then(|v| v.as_str());
        if let (Some(orig), Some(curr)) = (original, current) {
            let sim = if orig.len().min(curr.len()) > 0 && orig[..50.min(orig.len())] == curr[..50.min(curr.len())] { 1.0 } else { 0.3 };
            if sim < self.similarity_threshold {
                vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                    severity: if sim < 0.2 { Severity::Error } else { Severity::Warning },
                    category: IssueCategory::GoalDrift { similarity_to_original: sim },
                    description: format!("Agent drifted from goal (similarity: {:.0}%)", sim*100.0),
                    confidence: 1.0 - sim,
                    suggested_actions: vec!["nudge".into(), "interject".into()],
                    evidence_summary: format!("Task similarity dropped to {:.0}%", sim*100.0) }]
            } else { vec![] }
        } else { vec![] }
    }
}
