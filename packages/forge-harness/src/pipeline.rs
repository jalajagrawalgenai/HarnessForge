// forge-harness/src/pipeline.rs — Observe→Detect→Strategy→Action→Audit loop

use std::sync::Arc;
use std::time::Instant;
use forge_sdk::events::{AgentEvent, Intervention};
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;

use crate::event_bus::EventBus;
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
}

impl Pipeline {
    pub fn new(registry: Arc<PluginRegistry>, dry_run: bool) -> Self {
        Self {
            registry,
            bus: EventBus::new(500),
            cycles: Vec::new(),
            dry_run,
        }
    }

    /// Run one pipeline cycle against recent events.
    /// Returns interventions that should be applied.
    pub async fn cycle(&mut self, agent_id: &str) -> Vec<Intervention> {
        let start = Instant::now();
        let mut interventions = Vec::new();

        // ─── 1. OBSERVE ───
        let mut observations = Vec::new();
        for observer in self.registry.observers() {
            for event in self.bus.all_recent() {
                if let Some(obs) = observer.observe(event).await {
                    observations.push(obs);
                }
            }
        }

        // ─── 2. DETECT ───
        let mut detections = Vec::new();
        for detector in self.registry.detectors() {
            let found = detector.detect(agent_id, &observations).await;
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
                    break; // First matching strategy wins
                }
            }
        }

        // ─── 4. ACTION (interventions collected, applied by runtime) ───

        // ─── 5. AUDIT (record the cycle) ───
        self.cycles.push(PipelineCycle {
            observations,
            detections,
            selected_strategies: selected,
            intervention_applied: interventions.last().cloned(),
            duration_us: start.elapsed().as_micros() as u64,
        });

        interventions
    }

    /// Feed events into the pipeline
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
