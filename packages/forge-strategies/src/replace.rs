use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct ReplaceStrategy;

#[async_trait]
impl Strategy for ReplaceStrategy {
    fn name(&self) -> &'static str {
        "replace"
    }
    fn priority(&self) -> u32 {
        45
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let intervention = Intervention::Replace {
            context_summary: format!("Previous agent failed: {}", detection.description),
            new_model: Some("claude-opus-4-8".into()),
        };
        Some(StrategyResult {
            strategy_name: "replace".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!("Replacing failed agent: {}", detection.description),
            confidence: detection.confidence * 0.85,
        })
    }
}
