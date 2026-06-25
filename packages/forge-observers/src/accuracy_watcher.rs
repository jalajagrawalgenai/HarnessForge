use async_trait::async_trait;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;
use std::sync::Mutex;

pub struct AccuracyWatcher {
    tests_passed: Mutex<u64>,
    tests_failed: Mutex<u64>,
    lint_errors: Mutex<u64>,
}

#[allow(clippy::new_without_default)]
impl AccuracyWatcher {
    pub fn new() -> Self {
        Self {
            tests_passed: Mutex::new(0),
            tests_failed: Mutex::new(0),
            lint_errors: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Observer for AccuracyWatcher {
    fn name(&self) -> &'static str {
        "accuracy"
    }
    fn dimension(&self) -> &'static str {
        "accuracy"
    }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::ToolCallEnd { result, .. } => {
                if result.content.contains("passed") && result.content.contains("test") {
                    *self.tests_passed.lock().unwrap() += 1;
                }
                if result.content.contains("FAILED") {
                    *self.tests_failed.lock().unwrap() += 1;
                }
                if result.content.contains("error") || result.content.contains("warning") {
                    *self.lint_errors.lock().unwrap() += 1;
                }
                let passed = *self.tests_passed.lock().unwrap();
                let failed = *self.tests_failed.lock().unwrap();
                let total = passed + failed;
                let rate = if total == 0 {
                    1.0
                } else {
                    passed as f64 / total as f64
                };
                Some(
                    serde_json::json!({"dimension":"accuracy","test_pass_rate":rate,"lint_errors":*self.lint_errors.lock().unwrap()}),
                )
            }
            _ => None,
        }
    }
}
