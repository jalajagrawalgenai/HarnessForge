use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct ForkStrategy;

#[async_trait]
impl Strategy for ForkStrategy {
    fn name(&self) -> &'static str { "fork" }
    fn priority(&self) -> u32 { 18 }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        Some(StrategyResult { strategy_name: "fork".into(),
            intervention: Intervention::Fork { count: 2, subtasks: vec!["implement".into(), "verify".into()] },
            priority: self.priority(),
            reasoning: format!("Forking for parallel execution: {}", detection.description),
            confidence: detection.confidence * 0.7 })
    }
}
