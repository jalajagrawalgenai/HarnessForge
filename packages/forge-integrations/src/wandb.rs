use serde_json::json;
use forge_sdk::types::audit::AuditEvent;

#[allow(dead_code)]
pub struct WandBExporter { api_key: String, project: String }

impl WandBExporter {
    pub fn new(key: &str, project: &str) -> Self { Self { api_key: key.into(), project: project.into() } }
    pub fn export_session(&self, session_id: &str, events: &[AuditEvent]) -> String {
        let calls = events.iter().map(|e| json!({
            "trace_id": session_id, "span_name": e.event_type,
            "start_time": e.created_at.timestamp_millis(),
            "attributes": { "phase": format!("{:?}", e.phase), "sequence": e.sequence }
        })).collect::<Vec<_>>();
        json!({"project":self.project,"calls":calls}).to_string()
    }
}
