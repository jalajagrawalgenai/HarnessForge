use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct MemoryWatcher { retrievals: Mutex<u64>, hits: Mutex<u64> }

impl MemoryWatcher { pub fn new() -> Self { Self { retrievals: Mutex::new(0), hits: Mutex::new(0) } } }

#[async_trait]
impl Observer for MemoryWatcher {
    fn name(&self) -> &'static str { "memory" }
    fn dimension(&self) -> &'static str { "memory" }
    async fn observe(&self, _event: &AgentEvent) -> Option<serde_json::Value> {
        let ret = *self.retrievals.lock().unwrap();
        let hit = *self.hits.lock().unwrap();
        let rate = if ret == 0 { 0.0 } else { hit as f64 / ret as f64 };
        Some(serde_json::json!({"dimension":"memory","hit_rate":rate}))
    }
}
