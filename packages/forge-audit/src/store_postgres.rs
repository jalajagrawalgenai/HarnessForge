use async_trait::async_trait;
use forge_sdk::error::ForgeError;
use forge_sdk::traits::store::AuditStore;
use forge_sdk::types::audit::{AuditEvent, AuditReport, Checkpoint};
use sqlx::postgres::PgPool;

pub struct PostgresAuditStore { pool: PgPool }

impl PostgresAuditStore {
    pub async fn new(url: &str) -> Result<Self, ForgeError> {
        let pool = PgPool::connect(url).await.map_err(|e| ForgeError::Audit(e.to_string()))?;
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS audit_events (
                id BIGSERIAL PRIMARY KEY, session_id UUID NOT NULL, agent_id UUID,
                trace_id UUID NOT NULL, sequence BIGINT NOT NULL, phase TEXT NOT NULL,
                event_type TEXT NOT NULL, event_data JSONB NOT NULL,
                parent_event BIGINT, checkpoint_ref UUID, hash_chain TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                search_text TEXT, dimension TEXT, severity TEXT,
                detector_name TEXT, strategy_name TEXT, tool_name TEXT, model_id TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_audit_session ON audit_events(session_id);
            CREATE INDEX IF NOT EXISTS idx_audit_created ON audit_events(created_at DESC);
        "#).execute(&pool).await.map_err(|e| ForgeError::Audit(e.to_string()))?;
        Ok(Self { pool })
    }
}

#[async_trait]
impl AuditStore for PostgresAuditStore {
    async fn append(&self, event: &AuditEvent) -> Result<i64, ForgeError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO audit_events (session_id, agent_id, trace_id, sequence, phase, event_type, event_data, hash_chain, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) RETURNING id"
        )
        .bind(event.session_id).bind(event.agent_id).bind(event.trace_id).bind(event.sequence)
        .bind(format!("{:?}", event.phase)).bind(&event.event_type)
        .bind(&event.event_data).bind(&event.hash_chain).bind(event.created_at)
        .fetch_one(&self.pool).await.map_err(|e| ForgeError::Audit(e.to_string()))?;
        Ok(row.0)
    }
    async fn query(&self, _session_id: Option<&str>, _phase: Option<&str>, _limit: Option<u32>, _offset: Option<u32>) -> Result<Vec<AuditEvent>, ForgeError> {
        Ok(Vec::new()) // Stub — full impl would use sqlx::query_as
    }
    async fn search(&self, query: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        let _rows: Vec<(i64,)> = sqlx::query_as("SELECT id FROM audit_events WHERE search_text ILIKE $1 LIMIT 100")
            .bind(format!("%{}%", query)).fetch_all(&self.pool).await.map_err(|e| ForgeError::Audit(e.to_string()))?;
        Ok(Vec::new())
    }
    async fn get_report(&self, _sid: &str) -> Result<AuditReport, ForgeError> { Err(ForgeError::Audit("Not implemented".into())) }
    async fn save_checkpoint(&self, _cp: &Checkpoint) -> Result<(), ForgeError> { Ok(()) }
    async fn load_checkpoint(&self, _cid: &str) -> Result<Option<Checkpoint>, ForgeError> { Ok(None) }
    async fn replay_session(&self, _sid: &str) -> Result<Vec<AuditEvent>, ForgeError> { Ok(Vec::new()) }
}
