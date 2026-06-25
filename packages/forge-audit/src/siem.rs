use forge_sdk::types::audit::AuditEvent;
use serde_json::json;

pub enum SiemFormat { Splunk, Elastic, Datadog, Sumo }

pub fn export_for_siem(events: &[AuditEvent], format: SiemFormat) -> Vec<String> {
    events.iter().map(|e| {
        match format {
            SiemFormat::Splunk => json!({"time":e.created_at.to_rfc3339(),"event":e.event_type,"session":e.session_id.to_string()}).to_string(),
            SiemFormat::Elastic => json!({"@timestamp":e.created_at.to_rfc3339(),"message":e.event_type,"fields":{"session_id":e.session_id.to_string()}}).to_string(),
            SiemFormat::Datadog => json!({"timestamp":e.created_at.timestamp_millis(),"title":e.event_type,"text":serde_json::to_string(&e.event_data).unwrap_or_default()}).to_string(),
            SiemFormat::Sumo => format!("{} session={} phase={:?}", e.created_at.to_rfc3339(), e.session_id, e.phase),
        }
    }).collect()
}
