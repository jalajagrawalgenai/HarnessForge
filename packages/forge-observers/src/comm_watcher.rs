use async_trait::async_trait;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;
use std::collections::HashMap;
use std::sync::Mutex;

pub struct CommWatcher {
    msg_count: Mutex<HashMap<String, u64>>,
    total_msgs: Mutex<u64>,
    #[allow(dead_code)]
    last_msg_at: Mutex<Option<std::time::Instant>>,
}

#[allow(clippy::new_without_default)]
impl CommWatcher {
    pub fn new() -> Self {
        Self {
            msg_count: Mutex::new(HashMap::new()),
            total_msgs: Mutex::new(0),
            last_msg_at: Mutex::new(None),
        }
    }
}

#[async_trait]
impl Observer for CommWatcher {
    fn name(&self) -> &'static str {
        "comm"
    }
    fn dimension(&self) -> &'static str {
        "communication"
    }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::MessageSent { from, .. } = event {
            *self
                .msg_count
                .lock()
                .unwrap()
                .entry(from.clone())
                .or_default() += 1;
            *self.total_msgs.lock().unwrap() += 1;
        }
        let total = *self.total_msgs.lock().unwrap();
        Some(
            serde_json::json!({"dimension":"comm","total_messages":total,"participants":self.msg_count.lock().unwrap().len()}),
        )
    }
}
