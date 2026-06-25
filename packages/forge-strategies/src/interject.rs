use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct InterjectStrategy;

#[async_trait]
impl Strategy for InterjectStrategy {
    fn name(&self) -> &'static str {
        "interject"
    }
    fn priority(&self) -> u32 {
        35
    }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let intervention = Intervention::Interject {
            message: format!(
                "STOP. The harness detected: {}. Change your approach immediately.",
                detection.description
            ),
            reason: detection.description.clone(),
        };
        Some(StrategyResult {
            strategy_name: "interject".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!("Strong interjection for: {}", detection.description),
            confidence: detection.confidence,
        })
    }
}
