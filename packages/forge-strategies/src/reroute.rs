use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct RerouteStrategy;

#[async_trait]
impl Strategy for RerouteStrategy {
    fn name(&self) -> &'static str {
        "reroute"
    }
    fn priority(&self) -> u32 {
        22
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        Some(StrategyResult {
            strategy_name: "reroute".into(),
            intervention: Intervention::Reroute {
                to_node: "verify".into(),
                reason: detection.description.clone(),
            },
            priority: self.priority(),
            reasoning: format!("Rerouting agent: {}", detection.description),
            confidence: detection.confidence,
        })
    }
}
