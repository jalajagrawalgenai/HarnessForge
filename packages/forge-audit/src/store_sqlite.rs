// forge-audit/src/store_sqlite.rs — SQLite-backed audit store

use async_trait::async_trait;
use forge_sdk::error::ForgeError;
use forge_sdk::traits::store::AuditStore;
use forge_sdk::types::audit::{AuditEvent, AuditReport, Checkpoint};
use sqlx::sqlite::SqlitePool;

pub struct SqliteAuditStore {
    pool: SqlitePool,
}

impl SqliteAuditStore {
    pub async fn new(path: &str) -> Result<Self, ForgeError> {
        let pool = SqlitePool::connect(path)
            .await
            .map_err(|e| ForgeError::Audit(e.to_string()))?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                agent_id TEXT,
                trace_id TEXT NOT NULL,
                sequence INTEGER NOT NULL,
                phase TEXT NOT NULL,
                event_type TEXT NOT NULL,
                event_data TEXT NOT NULL,
                parent_event INTEGER,
                checkpoint_ref TEXT,
                hash_chain TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_audit_session ON audit_events(session_id);
            CREATE INDEX IF NOT EXISTS idx_audit_phase ON audit_events(phase);
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl AuditStore for SqliteAuditStore {
    async fn append(&self, event: &AuditEvent) -> Result<i64, ForgeError> {
        let result = sqlx::query(
            "INSERT INTO audit_events (session_id, agent_id, trace_id, sequence, phase, event_type, event_data, parent_event, checkpoint_ref, hash_chain, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(event.session_id.to_string())
        .bind(event.agent_id.map(|a| a.to_string()))
        .bind(event.trace_id.to_string())
        .bind(event.sequence)
        .bind(format!("{:?}", event.phase))
        .bind(&event.event_type)
        .bind(serde_json::to_string(&event.event_data).unwrap_or_default())
        .bind(event.parent_event)
        .bind(event.checkpoint_ref.map(|c| c.to_string()))
        .bind(&event.hash_chain)
        .bind(event.created_at.to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;

        Ok(result.last_insert_rowid())
    }

    async fn query(
        &self,
        _session_id: Option<&str>,
        _phase: Option<&str>,
        _limit: Option<u32>,
        _offset: Option<u32>,
    ) -> Result<Vec<AuditEvent>, ForgeError> {
        // Stub — full implementation would use sqlx::query_as
        Ok(Vec::new())
    }

    async fn search(&self, _query: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        Ok(Vec::new())
    }

    async fn get_report(&self, _session_id: &str) -> Result<AuditReport, ForgeError> {
        Err(ForgeError::Audit("Not implemented".into()))
    }

    async fn save_checkpoint(&self, _checkpoint: &Checkpoint) -> Result<(), ForgeError> {
        Ok(())
    }

    async fn load_checkpoint(&self, _checkpoint_id: &str) -> Result<Option<Checkpoint>, ForgeError> {
        Ok(None)
    }

    async fn replay_session(&self, _session_id: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        Ok(Vec::new())
    }
}
