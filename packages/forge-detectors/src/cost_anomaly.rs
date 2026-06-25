use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct CostAnomalyDetector { threshold_multiplier: f64 }

impl CostAnomalyDetector { pub fn new(t: f64) -> Self { Self { threshold_multiplier: t } } }

#[async_trait]
impl Detector for CostAnomalyDetector {
    fn name(&self) -> &'static str { "cost_anomaly" }
    fn description(&self) -> &'static str { "Detects cost spikes vs historical baseline" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let costs: Vec<f64> = obs.iter()
            .filter_map(|o| o.get("cost_per_turn").and_then(|v| v.as_f64())).collect();
        if costs.len() < 5 { return vec![]; }
        let avg: f64 = costs.iter().take(5).sum::<f64>() / 5.0;
        let last = *costs.last().unwrap();
        if avg > 0.0 && last > avg * self.threshold_multiplier {
            let mult = last / avg;
            vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                severity: if mult > 5.0 { Severity::Error } else { Severity::Warning },
                category: IssueCategory::CostAnomaly { expected_cost: avg, actual_cost: last, multiplier: mult },
                description: format!("Cost spike: ${:.4} vs avg ${:.4}", last, avg),
                confidence: ((mult - 1.0) / self.threshold_multiplier).min(1.0),
                suggested_actions: vec!["degrade".into(), "escalate".into()],
                evidence_summary: format!("Cost {:.1}x baseline", mult) }]
        } else { vec![] }
    }
}
