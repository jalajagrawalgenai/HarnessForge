use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct DiversityWatcher { approaches: Mutex<HashSet<String>> }

#[allow(clippy::new_without_default)]
impl DiversityWatcher { pub fn new() -> Self { Self { approaches: Mutex::new(HashSet::new()) } } }

#[async_trait]
impl Observer for DiversityWatcher {
    fn name(&self) -> &'static str { "diversity" }
    fn dimension(&self) -> &'static str { "diversity" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::OutputComplete { content, .. } = event {
            let key = content.chars().take(100).collect::<String>();
            self.approaches.lock().unwrap().insert(key);
        }
        let count = self.approaches.lock().unwrap().len();
        Some(serde_json::json!({"dimension":"diversity","unique_approaches":count}))
    }
}
