// forge-audit/src/store_sqlite.rs — SQLite-backed audit store

use async_trait::async_trait;
use forge_sdk::error::ForgeError;
use forge_sdk::traits::store::AuditStore;
use forge_sdk::types::audit::{
    AuditEvent, AuditPhase, AuditReport, Checkpoint, CheckpointSummary, DetectionSummary,
    InterventionSummary, ObservationSummary,
};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::str::FromStr;
use uuid::Uuid;

pub struct SqliteAuditStore {
    pool: SqlitePool,
}

impl SqliteAuditStore {
    pub async fn new(path: &str) -> Result<Self, ForgeError> {
        let pool = SqlitePool::connect(path)
            .await
            .map_err(|e| ForgeError::Audit(e.to_string()))?;

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
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                event_id INTEGER NOT NULL,
                session_id TEXT NOT NULL,
                agent_states TEXT NOT NULL,
                context_snapshot TEXT,
                message_queue TEXT,
                state_store TEXT,
                graph_state TEXT,
                task_progress TEXT,
                plan TEXT,
                token_usage TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;

        Ok(Self { pool })
    }

    fn row_to_event(row: &sqlx::sqlite::SqliteRow) -> Result<AuditEvent, ForgeError> {
        let id: i64 = row.get(0);
        let session_id_str: String = row.get(1);
        let agent_id_str: Option<String> = row.get(2);
        let trace_id_str: String = row.get(3);
        let sequence: i64 = row.get(4);
        let phase_str: String = row.get(5);
        let event_type: String = row.get(6);
        let event_data_str: String = row.get(7);
        let parent_event: Option<i64> = row.get(8);
        let checkpoint_ref_str: Option<String> = row.get(9);
        let hash_chain: String = row.get(10);
        let created_at_str: String = row.get(11);

        Ok(AuditEvent {
            id,
            session_id: Uuid::from_str(&session_id_str).unwrap_or_else(|_| Uuid::nil()),
            agent_id: agent_id_str.and_then(|s| Uuid::from_str(&s).ok()),
            trace_id: Uuid::from_str(&trace_id_str).unwrap_or_else(|_| Uuid::nil()),
            sequence,
            phase: match phase_str.as_str() {
                "Observe" => AuditPhase::Observe,
                "Detect" => AuditPhase::Detect,
                "Strategy" => AuditPhase::Strategy,
                "Action" => AuditPhase::Action,
                _ => AuditPhase::Observe,
            },
            event_type,
            event_data: serde_json::from_str(&event_data_str).unwrap_or_default(),
            parent_event,
            checkpoint_ref: checkpoint_ref_str.and_then(|s| Uuid::from_str(&s).ok()),
            hash_chain,
            created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now()),
        })
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
        session_id: Option<&str>,
        phase: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<AuditEvent>, ForgeError> {
        let mut sql = String::from(
            "SELECT id, session_id, agent_id, trace_id, sequence, phase, event_type, event_data, parent_event, checkpoint_ref, hash_chain, created_at FROM audit_events WHERE 1=1",
        );
        if session_id.is_some() {
            sql.push_str(" AND session_id = ?");
        }
        if phase.is_some() {
            sql.push_str(" AND LOWER(phase) = LOWER(?)");
        }
        sql.push_str(" ORDER BY sequence ASC");
        if let Some(lim) = limit {
            sql.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            sql.push_str(&format!(" OFFSET {}", off));
        }

        let mut q = sqlx::query(&sql);
        if let Some(sid) = session_id {
            q = q.bind(sid);
        }
        if let Some(p) = phase {
            q = q.bind(p);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        rows.iter().map(Self::row_to_event).collect()
    }

    async fn search(&self, query_str: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        let pattern = format!("%{}%", query_str);
        let rows = sqlx::query(
            "SELECT id, session_id, agent_id, trace_id, sequence, phase, event_type, event_data, parent_event, checkpoint_ref, hash_chain, created_at FROM audit_events WHERE event_type LIKE ? OR event_data LIKE ? OR session_id LIKE ? ORDER BY sequence ASC LIMIT 100",
        )
        .bind(&pattern).bind(&pattern).bind(&pattern)
        .fetch_all(&self.pool).await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;
        rows.iter().map(Self::row_to_event).collect()
    }

    async fn get_report(&self, session_id: &str) -> Result<AuditReport, ForgeError> {
        let events = self.replay_session(session_id).await?;
        if events.is_empty() {
            return Err(ForgeError::Audit(format!(
                "No events for session {}",
                session_id
            )));
        }
        let sid = Uuid::from_str(session_id).unwrap_or_else(|_| Uuid::nil());
        let first = events.first().unwrap();
        let last = events.last().unwrap();
        let duration = (last.created_at - first.created_at).num_milliseconds() as f64 / 1000.0;

        let mut dims: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for e in events.iter().filter(|e| e.phase == AuditPhase::Observe) {
            if let Some(d) = e.event_data.get("dimension").and_then(|v| v.as_str()) {
                *dims.entry(d.to_string()).or_insert(0) += 1;
            }
        }
        let observations: Vec<ObservationSummary> = dims
            .into_iter()
            .map(|(dim, count)| ObservationSummary {
                dimension: dim,
                event_count: count,
            })
            .collect();
        let detections: Vec<DetectionSummary> = events
            .iter()
            .filter(|e| e.phase == AuditPhase::Detect)
            .map(|e| DetectionSummary {
                turn: 0,
                detector: e.event_type.clone(),
                category: e
                    .event_data
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .into(),
                severity: e
                    .event_data
                    .get("severity")
                    .and_then(|v| v.as_str())
                    .unwrap_or("info")
                    .into(),
                confidence: e
                    .event_data
                    .get("confidence")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            })
            .collect();
        let interventions: Vec<InterventionSummary> = events
            .iter()
            .filter(|e| e.phase == AuditPhase::Action)
            .map(|e| InterventionSummary {
                turn: 0,
                strategy: e.event_type.clone(),
                outcome: "applied".into(),
                impact: None,
            })
            .collect();
        let cp_rows = sqlx::query("SELECT id FROM checkpoints WHERE session_id = ?")
            .bind(session_id)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let checkpoints: Vec<CheckpointSummary> = cp_rows
            .iter()
            .map(|_| CheckpointSummary {
                turn: 0,
                reason: "saved".into(),
            })
            .collect();
        let total_tokens: u64 = events
            .iter()
            .filter_map(|e| {
                e.event_data
                    .get("token_count")
                    .or_else(|| e.event_data.get("total_tokens"))
                    .and_then(|v| v.as_u64())
            })
            .sum();

        Ok(AuditReport {
            session_id: sid,
            task: String::new(),
            agent_type: String::new(),
            model: String::new(),
            duration_secs: duration,
            total_tokens,
            total_cost: 0.0,
            health_score: None,
            observations,
            detections,
            interventions,
            checkpoints,
            harness_effectiveness: None,
        })
    }

    async fn save_checkpoint(&self, cp: &Checkpoint) -> Result<(), ForgeError> {
        sqlx::query(
            "INSERT OR REPLACE INTO checkpoints (id, event_id, session_id, agent_states, context_snapshot, message_queue, state_store, graph_state, task_progress, plan, token_usage, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(cp.id.to_string()).bind(cp.event_id).bind(cp.session_id.to_string())
        .bind(serde_json::to_string(&cp.agent_states).unwrap_or_default())
        .bind(cp.context_snapshot.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(cp.message_queue.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(cp.state_store.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(cp.graph_state.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(cp.task_progress.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(cp.plan.as_ref().map(|v| serde_json::to_string(v).unwrap_or_default()))
        .bind(serde_json::to_string(&cp.token_usage).unwrap_or_default())
        .bind(cp.created_at.to_rfc3339())
        .execute(&self.pool).await.map_err(|e| ForgeError::Audit(e.to_string()))?;
        Ok(())
    }

    async fn load_checkpoint(&self, checkpoint_id: &str) -> Result<Option<Checkpoint>, ForgeError> {
        let row = sqlx::query(
            "SELECT id, event_id, session_id, agent_states, context_snapshot, message_queue, state_store, graph_state, task_progress, plan, token_usage, created_at FROM checkpoints WHERE id = ?",
        )
        .bind(checkpoint_id).fetch_optional(&self.pool).await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;

        match row {
            None => Ok(None),
            Some(r) => {
                let id_str: String = r.get(0);
                let id = Uuid::from_str(&id_str).unwrap_or_else(|_| Uuid::nil());
                let event_id: i64 = r.get(1);
                let sid_str: String = r.get(2);
                let session_id = Uuid::from_str(&sid_str).unwrap_or_else(|_| Uuid::nil());
                let as_str: String = r.get(3);
                let cs_str: Option<String> = r.get(4);
                let mq_str: Option<String> = r.get(5);
                let ss_str: Option<String> = r.get(6);
                let gs_str: Option<String> = r.get(7);
                let tp_str: Option<String> = r.get(8);
                let pl_str: Option<String> = r.get(9);
                let tu_str: String = r.get(10);
                let ca_str: String = r.get(11);

                Ok(Some(Checkpoint {
                    id,
                    event_id,
                    session_id,
                    agent_states: serde_json::from_str(&as_str).unwrap_or_default(),
                    context_snapshot: cs_str.and_then(|s| serde_json::from_str(&s).ok()),
                    message_queue: mq_str.and_then(|s| serde_json::from_str(&s).ok()),
                    state_store: ss_str.and_then(|s| serde_json::from_str(&s).ok()),
                    graph_state: gs_str.and_then(|s| serde_json::from_str(&s).ok()),
                    task_progress: tp_str.and_then(|s| serde_json::from_str(&s).ok()),
                    plan: pl_str.and_then(|s| serde_json::from_str(&s).ok()),
                    token_usage: serde_json::from_str(&tu_str).unwrap_or_default(),
                    created_at: chrono::DateTime::parse_from_rfc3339(&ca_str)
                        .map(|dt| dt.with_timezone(&chrono::Utc))
                        .unwrap_or_else(|_| chrono::Utc::now()),
                }))
            }
        }
    }

    async fn replay_session(&self, session_id: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        let rows = sqlx::query(
            "SELECT id, session_id, agent_id, trace_id, sequence, phase, event_type, event_data, parent_event, checkpoint_ref, hash_chain, created_at FROM audit_events WHERE session_id = ? ORDER BY sequence ASC",
        )
        .bind(session_id).fetch_all(&self.pool).await
        .map_err(|e| ForgeError::Audit(e.to_string()))?;
        rows.iter().map(Self::row_to_event).collect()
    }
}
