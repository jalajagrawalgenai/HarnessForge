use forge_sdk::types::audit::AuditEvent;

pub struct SearchEngine;

impl SearchEngine {
    pub fn search<'a>(events: &'a [AuditEvent], query: &str) -> Vec<&'a AuditEvent> {
        let lower = query.to_lowercase();
        events.iter().filter(|e| {
            e.event_type.to_lowercase().contains(&lower)
                || serde_json::to_string(&e.event_data).unwrap_or_default().to_lowercase().contains(&lower)
        }).collect()
    }

    pub fn by_detector<'a>(events: &'a [AuditEvent], detector: &str) -> Vec<&'a AuditEvent> {
        events.iter().filter(|e| e.event_type == detector).collect()
    }

    pub fn by_phase<'a>(events: &'a [AuditEvent], phase: &str) -> Vec<&'a AuditEvent> {
        events.iter().filter(|e| format!("{:?}", e.phase).to_lowercase() == phase.to_lowercase()).collect()
    }
}
