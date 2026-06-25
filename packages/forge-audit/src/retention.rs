use chrono::{DateTime, Duration, Utc};
use forge_sdk::types::audit::AuditEvent;

#[derive(Debug, Clone)]
pub enum RetentionPolicy { KeepAll, KeepDays(u32), KeepUntilStorageLimit(u64) }

pub struct RetentionManager { policy: RetentionPolicy }

impl RetentionManager {
    pub fn new(policy: RetentionPolicy) -> Self { Self { policy } }
    pub fn should_retain(&self, event: &AuditEvent, max_age_days: u32) -> bool {
        match &self.policy {
            RetentionPolicy::KeepAll => true,
            RetentionPolicy::KeepDays(days) => {
                let cutoff = Utc::now() - Duration::days(*days as i64);
                event.created_at > cutoff
            }
            RetentionPolicy::KeepUntilStorageLimit(_limit) => true,
        }
    }
    pub fn filter(&self, events: &[AuditEvent]) -> Vec<&AuditEvent> {
        match &self.policy {
            RetentionPolicy::KeepAll => events.iter().collect(),
            RetentionPolicy::KeepDays(days) => {
                let cutoff = Utc::now() - Duration::days(*days as i64);
                events.iter().filter(|e| e.created_at > cutoff).collect()
            }
            RetentionPolicy::KeepUntilStorageLimit(_) => events.iter().collect(),
        }
    }
}
