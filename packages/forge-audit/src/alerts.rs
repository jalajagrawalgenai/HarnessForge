use forge_sdk::types::detection::{DetectedIssue, Severity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub detector: String,
    pub min_severity: Severity,
    pub channels: Vec<String>,
    pub throttle_secs: u64,
}

pub struct AlertEngine {
    rules: Vec<AlertRule>,
}

impl AlertEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { rules: Vec::new() }
    }
    pub fn add_rule(&mut self, rule: AlertRule) {
        self.rules.push(rule);
    }
    pub fn evaluate(&self, issue: &DetectedIssue) -> Vec<String> {
        self.rules
            .iter()
            .filter(|r| r.detector == issue.category_name() && issue.severity >= r.min_severity)
            .flat_map(|r| r.channels.clone())
            .collect()
    }
}
