use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, Severity};
use forge_sdk::types::strategy::StrategyResult;
use uuid::Uuid;

pub struct PauseStrategy;

#[async_trait]
impl Strategy for PauseStrategy {
    fn name(&self) -> &'static str {
        "pause"
    }
    fn priority(&self) -> u32 {
        30
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        if detection.severity != Severity::Critical && detection.confidence < 0.9 {
            return None;
        }
        let intervention = Intervention::Pause {
            reason: detection.description.clone(),
            checkpoint_id: Uuid::new_v4(),
        };
        Some(StrategyResult {
            strategy_name: "pause".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!("Pausing for human review: {}", detection.description),
            confidence: detection.confidence,
        })
    }
}
