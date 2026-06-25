// forge-harness/src/pipeline.rs — Observe→Detect→Strategy→Action→Audit loop

use std::sync::Arc;
use std::time::Instant;
use forge_sdk::events::{AgentEvent, Intervention};
use forge_sdk::traits::store::AuditStore;
use forge_sdk::types::audit::{AuditEvent, AuditPhase, Checkpoint};
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

use crate::event_bus::EventBus;
use crate::human_gate::HumanGate;
use crate::plugin_registry::PluginRegistry;

/// A single cycle of the harness pipeline
#[derive(Debug)]
pub struct PipelineCycle {
    pub observations: Vec<serde_json::Value>,
    pub detections: Vec<DetectedIssue>,
    pub selected_strategies: Vec<StrategyResult>,
    pub intervention_applied: Option<Intervention>,
    pub duration_us: u64,
}

/// The main harness pipeline. Runs Observe→Detect→Strategy→Action per cycle.
pub struct Pipeline {
    registry: Arc<PluginRegistry>,
    bus: EventBus,
    cycles: Vec<PipelineCycle>,
    dry_run: bool,
    audit_store: Option<Arc<dyn AuditStore>>,
    human_gate: Option<HumanGate>,
}

impl Pipeline {
    pub fn new(registry: Arc<PluginRegistry>, dry_run: bool) -> Self {
        Self {
            registry,
            bus: EventBus::new(500),
            cycles: Vec::new(),
            dry_run,
            audit_store: None,
            human_gate: None,
        }
    }

    /// Attach an audit store for persistent event recording.
    pub fn with_audit(mut self, store: Arc<dyn AuditStore>) -> Self {
        self.audit_store = Some(store);
        self
    }

    /// Attach a human gate for pausing on critical detections.
    pub fn with_human_gate(mut self, gate: HumanGate) -> Self {
        self.human_gate = Some(gate);
        self
    }

    /// Access the human gate for external state inspection.
    pub fn human_gate(&self) -> Option<&HumanGate> {
        self.human_gate.as_ref()
    }

    pub fn human_gate_mut(&mut self) -> Option<&mut HumanGate> {
        self.human_gate.as_mut()
    }

    /// Run one pipeline cycle against recent events.
    /// Returns interventions that should be applied.
    pub async fn cycle(&mut self, agent_id: &str) -> Vec<Intervention> {
        let start = Instant::now();
        let mut interventions = Vec::new();
        let session_id = uuid::Uuid::new_v4(); // In production, pass real session ID

        // ─── 1. OBSERVE ───
        let mut observations = Vec::new();
        for observer in self.registry.observers() {
            for event in self.bus.all_recent() {
                if let Some(obs) = observer.observe(event).await {
                    // Record observation in audit store
                    if let Some(ref store) = self.audit_store {
                        let audit_event = AuditEvent {
                            id: 0,
                            session_id,
                            agent_id: None,
                            trace_id: uuid::Uuid::new_v4(),
                            sequence: self.cycles.len() as i64,
                            phase: AuditPhase::Observe,
                            event_type: format!("{}_{}", observer.name(), observer.dimension()),
                            event_data: obs.clone(),
                            parent_event: None,
                            checkpoint_ref: None,
                            hash_chain: String::new(),
                            created_at: chrono::Utc::now(),
                        };
                        let _ = store.append(&audit_event).await;
                    }
                    observations.push(obs);
                }
            }
        }

        // ─── 2. DETECT ───
        let mut detections = Vec::new();
        for detector in self.registry.detectors() {
            let found = detector.detect(agent_id, &observations).await;
            for d in &found {
                // Check human gate before proceeding
                if let Some(ref gate) = self.human_gate {
                    if gate.should_pause(&d.severity, 1.0, 1.0) {
                        // Gate would pause — record but don't auto-intervene
                        if let Some(ref store) = self.audit_store {
                            let audit_event = AuditEvent {
                                id: 0,
                                session_id,
                                agent_id: None,
                                trace_id: uuid::Uuid::new_v4(),
                                sequence: self.cycles.len() as i64,
                                phase: AuditPhase::Detect,
                                event_type: detector.name().to_string(),
                                event_data: serde_json::json!({
                                    "category": d.category_name(),
                                    "severity": format!("{:?}", d.severity),
                                    "confidence": d.confidence,
                                    "human_gate": "would_pause",
                                }),
                                parent_event: None,
                                checkpoint_ref: None,
                                hash_chain: String::new(),
                                created_at: chrono::Utc::now(),
                            };
                            let _ = store.append(&audit_event).await;
                        }
                    }
                }

                // Record detection in audit store
                if let Some(ref store) = self.audit_store {
                    let audit_event = AuditEvent {
                        id: 0,
                        session_id,
                        agent_id: None,
                        trace_id: uuid::Uuid::new_v4(),
                        sequence: self.cycles.len() as i64,
                        phase: AuditPhase::Detect,
                        event_type: detector.name().to_string(),
                        event_data: serde_json::json!({
                            "category": d.category_name(),
                            "severity": format!("{:?}", d.severity),
                            "confidence": d.confidence,
                        }),
                        parent_event: None,
                        checkpoint_ref: None,
                        hash_chain: String::new(),
                        created_at: chrono::Utc::now(),
                    };
                    let _ = store.append(&audit_event).await;
                }
            }
            detections.extend(found);
        }

        // ─── 3. STRATEGIZE ───
        let mut selected = Vec::new();
        for detection in &detections {
            for strategy in self.registry.strategies() {
                if let Some(result) = strategy.evaluate(detection).await {
                    selected.push(result.clone());
                    if !self.dry_run {
                        interventions.push(result.intervention.clone());
                    }

                    // Record strategy selection in audit store
                    if let Some(ref store) = self.audit_store {
                        let audit_event = AuditEvent {
                            id: 0,
                            session_id,
                            agent_id: None,
                            trace_id: uuid::Uuid::new_v4(),
                            sequence: self.cycles.len() as i64,
                            phase: AuditPhase::Strategy,
                            event_type: strategy.name().to_string(),
                            event_data: serde_json::json!({
                                "detection": detection.category_name(),
                                "result": format!("{:?}", result),
                                "dry_run": self.dry_run,
                            }),
                            parent_event: None,
                            checkpoint_ref: None,
                            hash_chain: String::new(),
                            created_at: chrono::Utc::now(),
                        };
                        let _ = store.append(&audit_event).await;
                    }
                    break; // First matching strategy wins
                }
            }
        }

        // ─── 4. ACTION & 5. AUDIT (record the cycle) ───
        if let Some(iv) = interventions.last() {
            if let Some(ref store) = self.audit_store {
                let audit_event = AuditEvent {
                    id: 0,
                    session_id,
                    agent_id: None,
                    trace_id: uuid::Uuid::new_v4(),
                    sequence: self.cycles.len() as i64,
                    phase: AuditPhase::Action,
                    event_type: "intervention_applied".to_string(),
                    event_data: serde_json::json!({
                        "intervention": format!("{:?}", iv),
                    }),
                    parent_event: None,
                    checkpoint_ref: None,
                    hash_chain: String::new(),
                    created_at: chrono::Utc::now(),
                };
                let _ = store.append(&audit_event).await;
            }
        }

        self.cycles.push(PipelineCycle {
            observations,
            detections,
            selected_strategies: selected,
            intervention_applied: interventions.last().cloned(),
            duration_us: start.elapsed().as_micros() as u64,
        });

        interventions
    }

    /// Feed events into the pipeline bus
    pub fn feed(&mut self, event: &AgentEvent) {
        self.bus.dispatch(event);
    }

    pub fn feed_batch(&mut self, events: &[AgentEvent]) {
        self.bus.dispatch_batch(events);
    }

    /// Get pipeline statistics
    pub fn stats(&self) -> PipelineStats {
        let total_detections: usize = self.cycles.iter().map(|c| c.detections.len()).sum();
        let total_interventions: usize = self
            .cycles
            .iter()
            .filter(|c| c.intervention_applied.is_some())
            .count();
        PipelineStats {
            cycles: self.cycles.len(),
            total_detections,
            total_interventions,
            avg_cycle_us: self
                .cycles
                .iter()
                .map(|c| c.duration_us)
                .sum::<u64>()
                / self.cycles.len().max(1) as u64,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PipelineStats {
    pub cycles: usize,
    pub total_detections: usize,
    pub total_interventions: usize,
    pub avg_cycle_us: u64,
}
