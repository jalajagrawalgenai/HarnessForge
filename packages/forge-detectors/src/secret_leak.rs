// SecretLeakDetector — detects API keys, tokens, credentials in agent output

use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

const CRITICAL_PATTERNS: &[(&str, &str)] = &[
    ("sk-ant-", "Anthropic API key"),
    ("sk-", "OpenAI API key"),
    ("AKIA", "AWS Access Key"),
    ("ghp_", "GitHub Personal Access Token"),
    ("xoxb-", "Slack Bot Token"),
    ("BEGIN RSA PRIVATE KEY", "RSA Private Key"),
    ("BEGIN OPENSSH PRIVATE KEY", "SSH Private Key"),
    ("--password", "Password in argument"),
    ("Bearer eyJ", "JWT Token in output"),
];

pub struct SecretLeakDetector;

#[async_trait]
impl Detector for SecretLeakDetector {
    fn name(&self) -> &'static str { "secret_leak" }
    fn description(&self) -> &'static str {
        "Detects API keys, tokens, passwords, and credentials in agent output"
    }

    async fn detect(
        &self,
        agent_id: &str,
        observations: &[serde_json::Value],
    ) -> Vec<DetectedIssue> {
        let mut issues = Vec::new();

        for obs in observations {
            let content = obs
                .get("content")
                .or_else(|| obs.get("output"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            for (pattern, secret_type) in CRITICAL_PATTERNS {
                if content.contains(pattern) {
                    issues.push(DetectedIssue {
                        id: Uuid::new_v4(),
                        agent_id: agent_id.to_string(),
                        severity: Severity::Critical,
                        category: IssueCategory::SecretLeak {
                            secret_type: secret_type.to_string(),
                        },
                        description: format!(
                            "Agent output contains a {} — possible secret leak",
                            secret_type
                        ),
                        confidence: 1.0,
                        suggested_actions: vec![
                            "circuit_break".into(),
                            "quarantine".into(),
                            "pause".into(),
                        ],
                        evidence_summary: format!("Pattern '{}' detected in output", pattern),
                    });
                }
            }
        }

        issues
    }
}
