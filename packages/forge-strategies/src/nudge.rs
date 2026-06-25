// NudgeStrategy — injects a hint into the agent's context

use async_trait::async_trait;
use forge_sdk::events::Intervention;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

pub struct NudgeStrategy {
    max_nudges: u32,
}

impl NudgeStrategy {
    pub fn new(max_nudges: u32) -> Self {
        Self { max_nudges }
    }

    fn craft_nudge(issue: &DetectedIssue) -> String {
        match &issue.category {
            forge_sdk::types::detection::IssueCategory::LoopDetected { tool_name, .. } => {
                format!(
                    "Note: You've called '{}' multiple times without progress. \
                     Consider trying a different approach or proceeding with \
                     what you've already found.",
                    tool_name
                )
            }
            forge_sdk::types::detection::IssueCategory::StaleContext { file_path, .. } => {
                format!(
                    "Note: '{}' is already in your context. You don't need to re-read it. \
                     Proceed with the information you already have.",
                    file_path
                )
            }
            forge_sdk::types::detection::IssueCategory::AccuracyRisk { .. } => {
                "Note: You've generated code but haven't run the tests yet. \
                 Run the tests before proceeding to the next step.".to_string()
            }
            _ => {
                format!(
                    "Note: The harness detected: {}. Consider adjusting your approach.",
                    issue.description
                )
            }
        }
    }
}

#[async_trait]
impl Strategy for NudgeStrategy {
    fn name(&self) -> &'static str { "nudge" }
    fn priority(&self) -> u32 { 10 } // Low priority — try gentle fix first

    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let message = Self::craft_nudge(detection);
        let intervention = Intervention::Nudge {
            message: message.clone(),
            reason: detection.description.clone(),
        };

        Some(StrategyResult {
            strategy_name: "nudge".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!("Injecting gentle hint for issue: {}", detection.description),
            confidence: detection.confidence * 0.8, // Nudges are gentle, confidence slightly discounted
        })
    }
}
