use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct RunawayCostDetector { acceleration_threshold: f64 }
impl RunawayCostDetector { pub fn new(t: f64) -> Self { Self { acceleration_threshold: t } } }

#[async_trait]
impl Detector for RunawayCostDetector {
    fn name(&self) -> &'static str { "runaway_cost" }
    fn description(&self) -> &'static str { "Detects accelerating cost (2nd derivative)" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let costs: Vec<f64> = obs.iter().filter_map(|o| o.get("cost_per_turn").and_then(|v| v.as_f64())).collect();
        if costs.len() < 6 { return vec![]; }
        let deltas: Vec<f64> = costs.windows(2).map(|w| w[1] - w[0]).collect();
        let accel: Vec<f64> = deltas.windows(2).map(|w| w[1] - w[0]).collect();
        let avg_accel = accel.iter().sum::<f64>() / accel.len() as f64;
        if avg_accel > self.acceleration_threshold {
            vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                severity: if avg_accel > self.acceleration_threshold * 2.0 { Severity::Error } else { Severity::Warning },
                category: IssueCategory::RunawayCost { acceleration: avg_accel },
                description: format!("Runaway cost: accelerating at ${:.4}/turn²", avg_accel),
                confidence: (avg_accel / self.acceleration_threshold).min(1.0),
                suggested_actions: vec!["degrade".into(), "circuit_break".into()],
                evidence_summary: format!("Cost acceleration: {:.4}", avg_accel) }]
        } else { vec![] }
    }
}
