use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct OrchWatcher { agent_count: Mutex<u32>, forks: Mutex<u32>, max_depth: Mutex<u32> }

impl OrchWatcher { pub fn new() -> Self { Self { agent_count: Mutex::new(1), forks: Mutex::new(0), max_depth: Mutex::new(0) } } }

#[async_trait]
impl Observer for OrchWatcher {
    fn name(&self) -> &'static str { "orch" }
    fn dimension(&self) -> &'static str { "orchestration" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::Forked { .. } => { *self.forks.lock().unwrap() += 1; *self.agent_count.lock().unwrap() += 1; }
            AgentEvent::Completed { .. } | AgentEvent::Failed { .. } => { *self.agent_count.lock().unwrap() = self.agent_count.lock().unwrap().saturating_sub(1); }
            _ => {}
        }
        Some(serde_json::json!({"dimension":"orch","active_agents":*self.agent_count.lock().unwrap(),"total_forks":*self.forks.lock().unwrap()}))
    }
}
