use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct LatencyWatcher { measurements: Mutex<Vec<f64>> }

impl LatencyWatcher { pub fn new() -> Self { Self { measurements: Mutex::new(Vec::new()) } } }

#[async_trait]
impl Observer for LatencyWatcher {
    fn name(&self) -> &'static str { "latency" }
    fn dimension(&self) -> &'static str { "latency" }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::ToolCallEnd { result, .. } = event {
            let mut m = self.measurements.lock().unwrap();
            m.push(result.duration_ms as f64);
            m.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let p50 = if m.is_empty() { 0.0 } else { m[m.len()/2] };
            let p95i = (m.len() as f64 * 0.95) as usize;
            let p95 = m.get(p95i).copied().unwrap_or(*m.last().unwrap_or(&0.0));
            Some(serde_json::json!({"dimension":"latency","p50_ms":p50,"p95_ms":p95,"count":m.len()}))
        } else { None }
    }
}
