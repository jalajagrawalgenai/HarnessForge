// forge-sdk/src/traits/store.rs — AuditStore trait

use crate::error::ForgeError;
use crate::types::audit::{AuditEvent, AuditReport, Checkpoint};
use async_trait::async_trait;

#[async_trait]
pub trait AuditStore: Send + Sync {
    async fn append(&self, event: &AuditEvent) -> Result<i64, ForgeError>;
    async fn query(
        &self,
        session_id: Option<&str>,
        phase: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<AuditEvent>, ForgeError>;
    async fn search(&self, query: &str) -> Result<Vec<AuditEvent>, ForgeError>;
    async fn get_report(&self, session_id: &str) -> Result<AuditReport, ForgeError>;
    async fn save_checkpoint(&self, checkpoint: &Checkpoint) -> Result<(), ForgeError>;
    async fn load_checkpoint(&self, checkpoint_id: &str) -> Result<Option<Checkpoint>, ForgeError>;
    async fn replay_session(&self, session_id: &str) -> Result<Vec<AuditEvent>, ForgeError>;
}
