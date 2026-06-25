use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory};
use forge_sdk::types::strategy::StrategyResult;

pub struct DiversifyStrategy;

#[async_trait]
impl Strategy for DiversifyStrategy {
    fn name(&self) -> &'static str { "diversify" }
    fn priority(&self) -> u32 { 20 }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        if !matches!(&detection.category, IssueCategory::VarietyCollapse { .. }) { return None; }
        let intervention = Intervention::Diversify {
            alternative_approach: "Try a completely different approach. MVP-first → Risk-first → User-first".into(),
        };
        Some(StrategyResult { strategy_name: "diversify".into(), intervention, priority: self.priority(),
            reasoning: format!("Forcing diverse strategies: {}", detection.description),
            confidence: detection.confidence })
    }
}
