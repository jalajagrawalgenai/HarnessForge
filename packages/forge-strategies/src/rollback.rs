use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;
use uuid::Uuid;

pub struct RollbackStrategy;

#[async_trait]
impl Strategy for RollbackStrategy {
    fn name(&self) -> &'static str {
        "rollback"
    }
    fn priority(&self) -> u32 {
        28
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        Some(StrategyResult {
            strategy_name: "rollback".into(),
            intervention: Intervention::Rollback {
                checkpoint_id: Uuid::new_v4(),
                reason: detection.description.clone(),
            },
            priority: self.priority(),
            reasoning: format!("Rolling back: {}", detection.description),
            confidence: detection.confidence,
        })
    }
}
