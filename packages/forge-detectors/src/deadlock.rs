use std::collections::HashMap;
use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct DeadlockDetector { timeout_secs: u64 }

impl DeadlockDetector { pub fn new(t: u64) -> Self { Self { timeout_secs: t } } }

#[async_trait]
impl Detector for DeadlockDetector {
    fn name(&self) -> &'static str { "deadlock" }
    fn description(&self) -> &'static str { "Detects circular wait between agents" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let mut waiting: HashMap<String, (String, u64)> = HashMap::new();
        for o in obs {
            if let (Some(a), Some(w), Some(d)) = (
                o.get("waiting_agent").and_then(|v| v.as_str()),
                o.get("waiting_for").and_then(|v| v.as_str()),
                o.get("wait_duration_secs").and_then(|v| v.as_u64()),
            ) { waiting.insert(a.into(), (w.into(), d)); }
        }
        let mut issues = Vec::new();
        for (agent, (waiting_for, dur)) in &waiting {
            if *dur >= self.timeout_secs {
                if let Some((other_for, _)) = waiting.get(waiting_for) {
                    if other_for == agent {
                        issues.push(DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                            severity: Severity::Error,
                            category: IssueCategory::Deadlock { agents: vec![agent.clone(), waiting_for.clone()], wait_duration_secs: *dur },
                            description: format!("Deadlock: {}↔{} ({}s)", agent, waiting_for, dur),
                            confidence: 1.0, suggested_actions: vec!["reroute".into()],
                            evidence_summary: format!("Circular wait {}s", dur) });
                    }
                }
            }
        }
        issues
    }
}
