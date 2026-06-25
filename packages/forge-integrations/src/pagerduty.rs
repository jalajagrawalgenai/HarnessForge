use serde_json::json;

pub struct PagerDutyIntegration {
    routing_key: String,
}
impl PagerDutyIntegration {
    pub fn new(key: &str) -> Self {
        Self {
            routing_key: key.into(),
        }
    }
    pub fn trigger(&self, title: &str, details: &str, severity: &str) -> String {
        json!({"routing_key":self.routing_key,"event_action":"trigger","payload":{"summary":title,"severity":severity,"source":"forge","custom_details":details}}).to_string()
    }
    pub fn resolve(&self, dedup: &str) -> String {
        json!({"routing_key":self.routing_key,"event_action":"resolve","dedup_key":dedup})
            .to_string()
    }
}
