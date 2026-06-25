use async_trait::async_trait;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use uuid::Uuid;

pub struct AccuracyRiskDetector;

#[async_trait]
impl Detector for AccuracyRiskDetector {
    fn name(&self) -> &'static str {
        "accuracy_risk"
    }
    fn description(&self) -> &'static str {
        "Detects generated code without tests or verification"
    }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let has_write = obs
            .iter()
            .any(|o| o.get("tool").and_then(|v| v.as_str()) == Some("write"));
        let has_test = obs.iter().any(|o| {
            let c = o.get("content").and_then(|v| v.as_str()).unwrap_or("");
            c.contains("test") || c.contains("cargo test") || c.contains("pytest")
        });
        if has_write && !has_test {
            vec![DetectedIssue {
                id: Uuid::new_v4(),
                agent_id: agent_id.into(),
                severity: Severity::Warning,
                category: IssueCategory::AccuracyRisk {
                    risk_factors: vec!["no_tests".into()],
                },
                description: "Code generated but no tests executed".into(),
                confidence: 0.8,
                suggested_actions: vec!["nudge".into()],
                evidence_summary: "WriteFile without subsequent test execution".into(),
            }]
        } else {
            vec![]
        }
    }
}
