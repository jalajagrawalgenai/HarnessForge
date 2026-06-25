use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct OrchWatcher {
    agent_count: Mutex<u32>,
    forks: Mutex<u32>,
    #[allow(dead_code)] max_depth: Mutex<u32>,
}

#[allow(clippy::new_without_default)]
impl OrchWatcher {
    pub fn new() -> Self {
        Self {
            agent_count: Mutex::new(1),
            forks: Mutex::new(0),
            max_depth: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Observer for OrchWatcher {
    fn name(&self) -> &'static str {
        "orch"
    }
    fn dimension(&self) -> &'static str {
        "orchestration"
    }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::Forked { .. } => {
                if let Ok(mut f) = self.forks.lock() { *f += 1; }
                if let Ok(mut a) = self.agent_count.lock() { *a += 1; }
            }
            AgentEvent::Completed { .. } | AgentEvent::Failed { .. } => {
                if let Ok(mut a) = self.agent_count.lock() {
                    *a = a.saturating_sub(1);
                }
            }
            _ => {}
        }
        let active = self.agent_count.lock().map(|a| *a).unwrap_or(0);
        let total_forks = self.forks.lock().map(|f| *f).unwrap_or(0);
        Some(serde_json::json!({
            "dimension": "orch",
            "active_agents": active,
            "total_forks": total_forks,
        }))
    }
}
