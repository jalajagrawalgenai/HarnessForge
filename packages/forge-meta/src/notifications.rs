use chrono::Utc;

#[derive(Debug, Clone)]
pub struct ImprovementNotification {
    pub version: String, pub improvement_pct: f64, pub p_value: f64,
    pub edits_applied: usize, pub timestamp: chrono::DateTime<Utc>,
}

pub trait Notifier: Send + Sync {
    fn notify(&self, notification: &ImprovementNotification);
}

pub struct CliNotifier;
impl Notifier for CliNotifier {
    fn notify(&self, n: &ImprovementNotification) {
        println!("🚀 Harness {} deployed! +{:.0}% pass rate (p={:.4}). {} edits applied.", n.version, n.improvement_pct, n.p_value, n.edits_applied);
    }
}

pub struct WebhookNotifier { url: String }
impl WebhookNotifier {
    pub fn new(url: &str) -> Self { Self { url: url.into() } }
}
impl Notifier for WebhookNotifier {
    fn notify(&self, n: &ImprovementNotification) {
        let payload = serde_json::json!({
            "text": format!("🚀 Harness {} deployed! +{:.0}% pass rate (p={:.4}). {} edits applied.", n.version, n.improvement_pct, n.p_value, n.edits_applied)
        });
        let _ = reqwest::blocking::Client::new().post(&self.url).json(&payload).send();
    }
}
