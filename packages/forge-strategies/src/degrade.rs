use async_trait::async_trait;
use forge_sdk::events::{DegradeLevel, Intervention};
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct DegradeStrategy;

#[async_trait]
impl Strategy for DegradeStrategy {
    fn name(&self) -> &'static str { "degrade" }
    fn priority(&self) -> u32 { 15 }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let intervention = Intervention::Degrade { level: DegradeLevel::Mild };
        Some(StrategyResult { strategy_name: "degrade".into(), intervention, priority: self.priority(),
            reasoning: format!("Degrading to reduce cost: {}", detection.description),
            confidence: detection.confidence })
    }
}
