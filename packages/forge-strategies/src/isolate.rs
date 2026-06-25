use async_trait::async_trait;
use forge_sdk::events::{Intervention, IsolationLevel};
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, Severity};
use forge_sdk::types::strategy::StrategyResult;

pub struct IsolateStrategy;

#[async_trait]
impl Strategy for IsolateStrategy {
    fn name(&self) -> &'static str { "isolate" }
    fn priority(&self) -> u32 { 50 }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        if detection.severity != Severity::Error && detection.severity != Severity::Critical { return None; }
        let level = if detection.severity == Severity::Critical { IsolationLevel::FullSandbox } else { IsolationLevel::ToolRestrict };
        let intervention = Intervention::Isolate { level, reason: detection.description.clone() };
        Some(StrategyResult { strategy_name: "isolate".into(), intervention, priority: self.priority(),
            reasoning: format!("Isolating agent: {}", detection.description),
            confidence: detection.confidence })
    }
}
