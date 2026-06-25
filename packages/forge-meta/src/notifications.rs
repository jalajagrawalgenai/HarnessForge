use chrono::Utc;

#[derive(Debug, Clone)]
pub struct ImprovementNotification {
    pub version: String, pub improvement_pct: f64, pub p_value: f64,
    pub edits_applied: usize, pub timestamp: chrono::DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, notification: &ImprovementNotification);
}

pub struct CliNotifier;
#[async_trait::async_trait]
impl Notifier for CliNotifier {
    async fn notify(&self, n: &ImprovementNotification) {
        println!("🚀 Harness {} deployed! +{:.0}% pass rate (p={:.4}). {} edits applied.", n.version, n.improvement_pct, n.p_value, n.edits_applied);
    }
}

pub struct WebhookNotifier { url: String }
impl WebhookNotifier {
    pub fn new(url: &str) -> Self { Self { url: url.into() } }
}
#[async_trait::async_trait]
impl Notifier for WebhookNotifier {
    async fn notify(&self, n: &ImprovementNotification) {
        let payload = serde_json::json!({
            "text": format!("🚀 Harness {} deployed! +{:.0}% pass rate (p={:.4}). {} edits applied.", n.version, n.improvement_pct, n.p_value, n.edits_applied)
        });
        let _ = reqwest::Client::new().post(&self.url).json(&payload).send().await;
    }
}
