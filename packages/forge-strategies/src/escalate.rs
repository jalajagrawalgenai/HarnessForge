use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory};
use forge_sdk::types::strategy::StrategyResult;

pub struct EscalateStrategy;

#[async_trait]
impl Strategy for EscalateStrategy {
    fn name(&self) -> &'static str {
        "escalate"
    }
    fn priority(&self) -> u32 {
        25
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let new_model = match &detection.category {
            IssueCategory::ModelMismatch {
                suggested_model, ..
            } => Some(suggested_model.clone()),
            _ => Some("claude-sonnet-4-6".into()),
        };
        let intervention = Intervention::Escalate {
            new_model,
            budget_increase: Some(50000),
            reason: detection.description.clone(),
        };
        Some(StrategyResult {
            strategy_name: "escalate".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!("Escalating resources for: {}", detection.description),
            confidence: detection.confidence,
        })
    }
}
