// forge-audit/src/store_memory.rs — In-memory audit store (full implementation)

use async_trait::async_trait;
use forge_sdk::error::ForgeError;
use forge_sdk::traits::store::AuditStore;
use forge_sdk::types::audit::{
    AuditEvent, AuditPhase, AuditReport, Checkpoint, CheckpointSummary, DetectionSummary,
    InterventionSummary, ObservationSummary,
};
use std::collections::HashMap;
use std::sync::Mutex;
#[allow(unused_imports)]
use uuid::Uuid;

/// Fully functional in-memory audit store implementing all AuditStore trait methods.
pub struct MemoryAuditStore {
    events: Mutex<Vec<AuditEvent>>,
    checkpoints: Mutex<Vec<Checkpoint>>,
}

impl MemoryAuditStore {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
            checkpoints: Mutex::new(Vec::new()),
        }
    }
}

impl Default for MemoryAuditStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AuditStore for MemoryAuditStore {
    async fn append(&self, event: &AuditEvent) -> Result<i64, ForgeError> {
        let mut events = self
            .events
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let id = events.len() as i64 + 1;
        let mut event = event.clone();
        event.id = id;
        events.push(event);
        Ok(id)
    }

    async fn query(
        &self,
        session_id: Option<&str>,
        phase: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<AuditEvent>, ForgeError> {
        let events = self
            .events
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let mut filtered: Vec<AuditEvent> = events
            .iter()
            .filter(|e| {
                let sid_match = session_id.is_none_or(|s| e.session_id.to_string() == s);
                let phase_match = phase
                    .is_none_or(|p| format!("{:?}", e.phase).to_lowercase() == p.to_lowercase());
                sid_match && phase_match
            })
            .cloned()
            .collect();
        let off = offset.unwrap_or(0) as usize;
        let lim = limit.unwrap_or(u32::MAX) as usize;
        if off > 0 {
            filtered.drain(..off.min(filtered.len()));
        }
        filtered.truncate(lim);
        Ok(filtered)
    }

    async fn search(&self, query: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        let events = self
            .events
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let q = query.to_lowercase();
        Ok(events
            .iter()
            .filter(|e| {
                e.event_type.to_lowercase().contains(&q)
                    || serde_json::to_string(&e.event_data)
                        .unwrap_or_default()
                        .to_lowercase()
                        .contains(&q)
                    || e.session_id.to_string().to_lowercase().contains(&q)
            })
            .cloned()
            .collect())
    }

    async fn get_report(&self, session_id: &str) -> Result<AuditReport, ForgeError> {
        let events = self
            .events
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let se: Vec<&AuditEvent> = events
            .iter()
            .filter(|e| e.session_id.to_string() == session_id)
            .collect();
        if se.is_empty() {
            return Err(ForgeError::Audit(format!(
                "No events for session {}",
                session_id
            )));
        }
        let first = se.first().unwrap();
        let last = se.last().unwrap();
        let duration = (last.created_at - first.created_at).num_milliseconds() as f64 / 1000.0;

        let mut dims: HashMap<String, u64> = HashMap::new();
        for e in se.iter().filter(|e| e.phase == AuditPhase::Observe) {
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

        let detections: Vec<DetectionSummary> = se
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

        let interventions: Vec<InterventionSummary> = se
            .iter()
            .filter(|e| e.phase == AuditPhase::Action)
            .map(|e| InterventionSummary {
                turn: 0,
                strategy: e.event_type.clone(),
                outcome: "applied".into(),
                impact: None,
            })
            .collect();

        let cps = self
            .checkpoints
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let checkpoints: Vec<CheckpointSummary> = cps
            .iter()
            .filter(|c| c.session_id.to_string() == session_id)
            .map(|_| CheckpointSummary {
                turn: 0,
                reason: "saved".into(),
            })
            .collect();

        let total_tokens: u64 = se
            .iter()
            .filter_map(|e| {
                e.event_data
                    .get("token_count")
                    .or_else(|| e.event_data.get("total_tokens"))
                    .and_then(|v| v.as_u64())
            })
            .sum();

        Ok(AuditReport {
            session_id: first.session_id,
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
        let mut cps = self
            .checkpoints
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        cps.push(cp.clone());
        Ok(())
    }

    async fn load_checkpoint(&self, cid: &str) -> Result<Option<Checkpoint>, ForgeError> {
        let cps = self
            .checkpoints
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        Ok(cps.iter().find(|c| c.id.to_string() == cid).cloned())
    }

    async fn replay_session(&self, session_id: &str) -> Result<Vec<AuditEvent>, ForgeError> {
        let events = self
            .events
            .lock()
            .map_err(|e| ForgeError::Audit(e.to_string()))?;
        let mut se: Vec<AuditEvent> = events
            .iter()
            .filter(|e| e.session_id.to_string() == session_id)
            .cloned()
            .collect();
        se.sort_by_key(|e| e.sequence);
        Ok(se)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn mk(sid: Uuid, seq: i64, phase: AuditPhase, et: &str) -> AuditEvent {
        AuditEvent {
            id: 0,
            session_id: sid,
            agent_id: None,
            trace_id: Uuid::new_v4(),
            sequence: seq,
            phase,
            event_type: et.into(),
            event_data: serde_json::json!({"dimension":"token","token_count":100}),
            parent_event: None,
            checkpoint_ref: None,
            hash_chain: format!("h{}", seq),
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_append_and_query() {
        let s = MemoryAuditStore::new();
        let sid = Uuid::new_v4();
        let id = s
            .append(&mk(sid, 1, AuditPhase::Observe, "t"))
            .await
            .unwrap();
        assert!(id > 0);
        let r = s
            .query(Some(&sid.to_string()), None, None, None)
            .await
            .unwrap();
        assert_eq!(r.len(), 1);
    }

    #[tokio::test]
    async fn test_search_finds_match() {
        let s = MemoryAuditStore::new();
        let sid = Uuid::new_v4();
        s.append(&mk(sid, 1, AuditPhase::Detect, "loop_detected"))
            .await
            .unwrap();
        s.append(&mk(sid, 2, AuditPhase::Detect, "secret_leak"))
            .await
            .unwrap();
        assert_eq!(s.search("loop").await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_get_report_aggregates() {
        let s = MemoryAuditStore::new();
        let sid = Uuid::new_v4();
        s.append(&mk(sid, 1, AuditPhase::Observe, "t"))
            .await
            .unwrap();
        s.append(&mk(sid, 2, AuditPhase::Detect, "d"))
            .await
            .unwrap();
        s.append(&mk(sid, 3, AuditPhase::Action, "a"))
            .await
            .unwrap();
        let r = s.get_report(&sid.to_string()).await.unwrap();
        assert_eq!(r.observations.len(), 1);
        assert_eq!(r.detections.len(), 1);
        assert_eq!(r.interventions.len(), 1);
    }

    #[tokio::test]
    async fn test_checkpoint_save_load() {
        let s = MemoryAuditStore::new();
        let cid = Uuid::new_v4();
        let cp = Checkpoint {
            id: cid,
            event_id: 1,
            session_id: Uuid::new_v4(),
            agent_states: serde_json::json!({}),
            context_snapshot: None,
            message_queue: None,
            state_store: None,
            graph_state: None,
            task_progress: None,
            plan: None,
            token_usage: serde_json::json!({"t": 500}),
            created_at: Utc::now(),
        };
        s.save_checkpoint(&cp).await.unwrap();
        assert!(s.load_checkpoint(&cid.to_string()).await.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_replay_session_sorted() {
        let s = MemoryAuditStore::new();
        let sid = Uuid::new_v4();
        for i in 1..=5 {
            s.append(&mk(sid, i, AuditPhase::Observe, &format!("e{}", i)))
                .await
                .unwrap();
        }
        let replay = s.replay_session(&sid.to_string()).await.unwrap();
        assert_eq!(replay.len(), 5);
        assert!(replay.windows(2).all(|w| w[0].sequence < w[1].sequence));
    }

    #[tokio::test]
    async fn test_query_pagination() {
        let s = MemoryAuditStore::new();
        let sid = Uuid::new_v4();
        for i in 1..=10 {
            s.append(&mk(sid, i, AuditPhase::Observe, "x"))
                .await
                .unwrap();
        }
        assert_eq!(
            s.query(Some(&sid.to_string()), None, Some(3), Some(5))
                .await
                .unwrap()
                .len(),
            3
        );
    }
}
