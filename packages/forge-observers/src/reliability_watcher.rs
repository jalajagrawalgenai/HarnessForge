use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct ReliabilityWatcher { errors: Mutex<u64>, successes: Mutex<u64>, #[allow(dead_code)] timeouts: Mutex<u64> }

#[allow(clippy::new_without_default)]
impl ReliabilityWatcher { pub fn new() -> Self { Self { errors: Mutex::new(0), successes: Mutex::new(0), timeouts: Mutex::new(0) } } }

#[async_trait]
impl Observer for ReliabilityWatcher {
    fn name(&self) -> &'static str { "reliability" }
    fn dimension(&self) -> &'static str { "reliability" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::ToolCallEnd { result, .. } => {
                if result.is_error { *self.errors.lock().unwrap() += 1; } else { *self.successes.lock().unwrap() += 1; }
            }
            AgentEvent::Failed { .. } => { *self.errors.lock().unwrap() += 1; }
            _ => {}
        }
        let errs = *self.errors.lock().unwrap();
        let succ = *self.successes.lock().unwrap();
        let total = errs + succ;
        let rate = if total == 0 { 0.0 } else { errs as f64 / total as f64 };
        Some(serde_json::json!({"dimension":"reliability","error_rate":rate,"total_ops":total}))
    }
}
