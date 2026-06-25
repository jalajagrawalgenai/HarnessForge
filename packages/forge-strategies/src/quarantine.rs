use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory};
use forge_sdk::types::strategy::StrategyResult;

pub struct QuarantineStrategy;

#[async_trait]
impl Strategy for QuarantineStrategy {
    fn name(&self) -> &'static str { "quarantine" }
    fn priority(&self) -> u32 { 40 }
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let should_quarantine = matches!(&detection.category,
            IssueCategory::SecretLeak { .. } | IssueCategory::ComplianceGap { .. });
        if !should_quarantine { return None; }
        let intervention = Intervention::Quarantine { reason: detection.description.clone() };
        Some(StrategyResult { strategy_name: "quarantine".into(), intervention, priority: self.priority(),
            reasoning: format!("Quarantining output: {}", detection.description),
            confidence: detection.confidence })
    }
}
