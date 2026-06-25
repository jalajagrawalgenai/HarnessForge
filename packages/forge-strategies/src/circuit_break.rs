// CircuitBreakStrategy — emergency stop all agents

use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, Severity};
use forge_sdk::types::strategy::StrategyResult;

pub struct CircuitBreakStrategy;

#[async_trait]
impl Strategy for CircuitBreakStrategy {
    fn name(&self) -> &'static str { "circuit_break" }
    fn priority(&self) -> u32 { 100 } // Highest priority — always wins if conditions met

    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        // Only circuit break on critical security/compliance issues
        let should_break = matches!(detection.severity, Severity::Critical)
            && matches!(
                &detection.category,
                forge_sdk::types::detection::IssueCategory::SecretLeak { .. }
                    | forge_sdk::types::detection::IssueCategory::ComplianceGap { .. }
            );

        if !should_break {
            return None;
        }

        let intervention = Intervention::CircuitBreak {
            reason: format!(
                "Critical issue detected: {} (confidence: {:.0}%)",
                detection.description,
                detection.confidence * 100.0
            ),
        };

        Some(StrategyResult {
            strategy_name: "circuit_break".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!(
                "Emergency stop triggered by {:?} severity {} issue",
                detection.severity, detection.description
            ),
            confidence: detection.confidence,
        })
    }
}
