use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct MemoryWatcher {
    retrievals: Mutex<u64>,
    hits: Mutex<u64>,
    store_count: Mutex<u64>,
}

#[allow(clippy::new_without_default)]
impl MemoryWatcher {
    pub fn new() -> Self {
        Self {
            retrievals: Mutex::new(0),
            hits: Mutex::new(0),
            store_count: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Observer for MemoryWatcher {
    fn name(&self) -> &'static str {
        "memory"
    }
    fn dimension(&self) -> &'static str {
        "memory"
    }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            // Track memory retrievals from reasoning/tool call events
            AgentEvent::ThinkingStart { .. } | AgentEvent::ToolCallStart { .. } => {
                if let Ok(mut r) = self.retrievals.lock() {
                    *r += 1;
                }
            }
            // Track successful tool completions as memory "hits"
            AgentEvent::ToolCallEnd { result, .. } => {
                if !result.is_error {
                    if let Ok(mut h) = self.hits.lock() { *h += 1; }
                }
            }
            // Track tool calls as memory store operations
            AgentEvent::OutputComplete { .. } => {
                if let Ok(mut s) = self.store_count.lock() { *s += 1; }
            }
            _ => {}
        }

        let ret = self.retrievals.lock().map(|r| *r).unwrap_or(0);
        let hit = self.hits.lock().map(|h| *h).unwrap_or(0);
        let stored = self.store_count.lock().map(|s| *s).unwrap_or(0);
        let hit_rate = if ret == 0 { 0.0 } else { hit as f64 / ret as f64 };
        let growth_rate_kb = stored.saturating_mul(4) as f64; // ~4KB per store

        Some(serde_json::json!({
            "dimension": "memory",
            "hit_rate": hit_rate,
            "retrievals": ret,
            "hits": hit,
            "stored": stored,
            "growth_rate_kb_hr": growth_rate_kb,
        }))
    }
}
