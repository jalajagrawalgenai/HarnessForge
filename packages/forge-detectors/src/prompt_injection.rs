use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

const PATTERNS: &[&str] = &["ignore previous instructions","ignore all previous","you are now DAN","system override","new instructions:","forget everything","pretend you are","disregard","override system prompt","bypass"];

pub struct PromptInjectionDetector;

#[async_trait]
impl Detector for PromptInjectionDetector {
    fn name(&self) -> &'static str { "prompt_injection" }
    fn description(&self) -> &'static str { "Detects attempts to override system instructions" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let mut issues = Vec::new();
        for o in obs {
            let text = o.get("content").and_then(|v| v.as_str()).unwrap_or("");
            let lower = text.to_lowercase();
            let hits: Vec<&&str> = PATTERNS.iter().filter(|p| lower.contains(&p.to_lowercase())).collect();
            if !hits.is_empty() {
                issues.push(DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                    severity: Severity::Error,
                    category: IssueCategory::PromptInjection { pattern_matched: hits.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(", ") },
                    description: format!("Prompt injection: '{}' pattern matched", hits[0]),
                    confidence: (hits.len() as f64 / 2.0).min(1.0),
                    suggested_actions: vec!["circuit_break".into(), "pause".into()],
                    evidence_summary: format!("{} patterns matched", hits.len()) });
            }
        }
        issues
    }
}
