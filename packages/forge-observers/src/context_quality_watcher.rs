use async_trait::async_trait;
use std::collections::HashSet;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct ContextQualityWatcher { files_read: Mutex<HashSet<String>>, total_reads: Mutex<u64>, redundant: Mutex<u64> }

#[allow(clippy::new_without_default)]
impl ContextQualityWatcher { pub fn new() -> Self { Self { files_read: Mutex::new(HashSet::new()), total_reads: Mutex::new(0), redundant: Mutex::new(0) } } }

#[async_trait]
impl Observer for ContextQualityWatcher {
    fn name(&self) -> &'static str { "context_quality" }
    fn dimension(&self) -> &'static str { "context_quality" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::ToolCallStart { tool, args, .. } = event {
            if tool == "read" || tool == "Read" {
                if let Some(path) = args.get("file_path").and_then(|v| v.as_str()) {
                    *self.total_reads.lock().unwrap() += 1;
                    let mut seen = self.files_read.lock().unwrap();
                    if seen.contains(path) { *self.redundant.lock().unwrap() += 1; }
                    seen.insert(path.to_string());
                }
            }
        }
        let total = *self.total_reads.lock().unwrap();
        let red = *self.redundant.lock().unwrap();
        let ratio = if total == 0 { 0.0 } else { red as f64 / total as f64 };
        Some(serde_json::json!({"dimension":"context_quality","redundancy_ratio":ratio,"unique_files":self.files_read.lock().unwrap().len()}))
    }
}
