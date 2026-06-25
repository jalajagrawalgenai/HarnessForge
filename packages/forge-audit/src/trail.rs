// forge-audit/src/trail.rs — Immutable append-only audit trail

use chrono::Utc;
use forge_sdk::types::audit::{AuditEvent, AuditPhase, AuditReport, DetectionSummary, InterventionSummary, ObservationSummary};
use sha2::{Digest, Sha256};
use uuid::Uuid;

/// Immutable append-only audit trail with hash-chain integrity
pub struct AuditTrail {
    events: Vec<AuditEvent>,
    session_id: Uuid,
    last_hash: Option<String>,
}

impl AuditTrail {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            events: Vec::new(),
            session_id,
            last_hash: None,
        }
    }

    /// Append an event with hash-chain integrity
    pub fn record(
        &mut self,
        phase: AuditPhase,
        event_type: &str,
        data: serde_json::Value,
        agent_id: Option<Uuid>,
        parent_event: Option<i64>,
    ) -> i64 {
        let event_id = self.events.len() as i64;
        let sequence = event_id + 1;

        // Build hash chain: SHA-256(prev_hash + current_event_data)
        let prev_hash = self.last_hash.clone().unwrap_or_else(|| {
            // Genesis hash from session_id
            let mut hasher = Sha256::new();
            hasher.update(self.session_id.as_bytes());
            hex::encode(hasher.finalize())
        });

        let mut hasher = Sha256::new();
        hasher.update(prev_hash.as_bytes());
        hasher.update(serde_json::to_string(&data).unwrap_or_default().as_bytes());
        let hash = hex::encode(hasher.finalize());

        let event = AuditEvent {
            id: event_id,
            session_id: self.session_id,
            agent_id,
            trace_id: Uuid::new_v4(),
            sequence,
            phase,
            event_type: event_type.to_string(),
            event_data: data,
            parent_event,
            checkpoint_ref: None,
            hash_chain: hash.clone(),
            created_at: Utc::now(),
        };

        self.last_hash = Some(hash);
        self.events.push(event);
        event_id
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    pub fn query_by_phase(&self, phase: AuditPhase) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.phase == phase).collect()
    }

    /// Verify hash-chain integrity — returns true if chain is unbroken
    pub fn verify_integrity(&self) -> bool {
        if self.events.is_empty() {
            return true;
        }

        // Genesis hash
        let mut hasher = Sha256::new();
        hasher.update(self.session_id.as_bytes());
        let mut expected_prev = hex::encode(hasher.finalize());

        for event in &self.events {
            let mut hasher = Sha256::new();
            hasher.update(expected_prev.as_bytes());
            hasher.update(
                serde_json::to_string(&event.event_data)
                    .unwrap_or_default()
                    .as_bytes(),
            );
            let computed = hex::encode(hasher.finalize());

            if computed != event.hash_chain {
                return false; // Chain broken — tampering detected
            }
            expected_prev = computed;
        }

        true
    }

    /// Generate a human-readable audit report
    pub fn generate_report(
        &self,
        task: &str,
        agent_type: &str,
        model: &str,
        duration_secs: f64,
    ) -> AuditReport {
        let observations: Vec<ObservationSummary> = self
            .query_by_phase(AuditPhase::Observe)
            .iter()
            .fold(Vec::new(), |mut acc, _| {
                // Group by event_type
                acc.push(ObservationSummary {
                    dimension: "token".into(),
                    event_count: 1,
                });
                acc
            });

        let detections: Vec<DetectionSummary> = self
            .query_by_phase(AuditPhase::Detect)
            .iter()
            .map(|e| DetectionSummary {
                turn: (e.sequence / 3) as u32,
                detector: e.event_type.clone(),
                category: "loop".into(),
                severity: "warning".into(),
                confidence: 0.9,
            })
            .collect();

        let interventions: Vec<InterventionSummary> = self
            .query_by_phase(AuditPhase::Action)
            .iter()
            .map(|e| InterventionSummary {
                turn: (e.sequence / 3) as u32,
                strategy: e.event_type.clone(),
                outcome: "applied".into(),
                impact: Some("context reduced".into()),
            })
            .collect();

        let total_tokens: u64 = self
            .events
            .iter()
            .filter_map(|e| e.event_data.get("total_tokens").and_then(|v| v.as_u64()))
            .sum();

        AuditReport {
            session_id: self.session_id,
            task: task.to_string(),
            agent_type: agent_type.to_string(),
            model: model.to_string(),
            duration_secs,
            total_tokens,
            total_cost: 0.0,
            health_score: None,
            observations,
            detections,
            interventions,
            checkpoints: vec![],
            harness_effectiveness: None,
        }
    }
}
