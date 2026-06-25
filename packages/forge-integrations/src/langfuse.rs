use forge_sdk::types::audit::AuditEvent;
use serde_json::json;

#[allow(dead_code)]
pub struct LangFuseExporter {
    public_key: String,
    secret_key: String,
    endpoint: String,
}

impl LangFuseExporter {
    pub fn new(pk: &str, sk: &str, ep: &str) -> Self {
        Self {
            public_key: pk.into(),
            secret_key: sk.into(),
            endpoint: ep.into(),
        }
    }
    pub fn export_session(&self, session_id: &str, events: &[AuditEvent]) -> String {
        let trace = json!({
            "id": session_id, "name": "forge-session",
            "observations": events.iter().map(|e| json!({
                "name": e.event_type, "type": "SPAN",
                "metadata": { "phase": format!("{:?}", e.phase), "sequence": e.sequence }
            })).collect::<Vec<_>>()
        });
        serde_json::to_string_pretty(&trace).unwrap_or_default()
    }
}
