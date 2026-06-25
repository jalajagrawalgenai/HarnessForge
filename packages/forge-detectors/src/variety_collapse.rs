use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct VarietyCollapseDetector { similarity_threshold: f64 }

impl VarietyCollapseDetector { pub fn new(t: f64) -> Self { Self { similarity_threshold: t } } }

#[async_trait]
impl Detector for VarietyCollapseDetector {
    fn name(&self) -> &'static str { "variety_collapse" }
    fn description(&self) -> &'static str { "Detects when agents produce near-identical outputs" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let outputs: Vec<&str> = obs.iter()
            .filter_map(|o| o.get("agent_output").and_then(|v| v.as_str())).collect();
        if outputs.len() < 3 { return vec![]; }
        let mut similar = 0u32;
        for i in 0..outputs.len() {
            for j in i+1..outputs.len() {
                if outputs[i].len().min(outputs[j].len()) > 0
                    && &outputs[i][..200.min(outputs[i].len())] == &outputs[j][..200.min(outputs[j].len())]
                { similar += 1; }
            }
        }
        let total = (outputs.len() * (outputs.len() - 1) / 2) as f64;
        let score = similar as f64 / total.max(1.0);
        if score > self.similarity_threshold {
            vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(), severity: Severity::Warning,
                category: IssueCategory::VarietyCollapse { similarity_score: score, agent_count: outputs.len() as u32 },
                description: format!("{} agents near-identical ({:.0}%)", outputs.len(), score*100.0),
                confidence: score, suggested_actions: vec!["diversify".into()],
                evidence_summary: format!("{}/{} pairs similar", similar, total as u32) }]
        } else { vec![] }
    }
}
