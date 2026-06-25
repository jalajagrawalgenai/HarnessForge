use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

const PII_PATTERNS: &[&str] = &["@", "credit card", "ssn", "social security", "passport"];

pub struct ComplianceWatcher { pii_exposures: Mutex<u64>, #[allow(dead_code)] gate_bypasses: Mutex<u64> }

#[allow(clippy::new_without_default)]
impl ComplianceWatcher { pub fn new() -> Self { Self { pii_exposures: Mutex::new(0), gate_bypasses: Mutex::new(0) } } }

#[async_trait]
impl Observer for ComplianceWatcher {
    fn name(&self) -> &'static str { "compliance" }
    fn dimension(&self) -> &'static str { "compliance" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::OutputComplete { content, .. } = event {
            let lower = content.to_lowercase();
            let hits: Vec<&&str> = PII_PATTERNS.iter().filter(|p| lower.contains(&p.to_lowercase())).collect();
            if !hits.is_empty() { *self.pii_exposures.lock().unwrap() += hits.len() as u64; }
        }
        Some(serde_json::json!({"dimension":"compliance","pii_exposures":*self.pii_exposures.lock().unwrap()}))
    }
}
