use std::sync::Arc;
use tokio::sync::mpsc;

use crate::agent::AgentAdapter;
use crate::error::ForgeError;
use crate::traits::detector::Detector;
use crate::traits::observer::Observer;
use crate::traits::store::AuditStore;
use crate::traits::strategy::Strategy;

#[derive(Debug, Clone)]
pub struct HarnessConfig {
    pub checkpoint_interval: u32,
    pub max_interventions: u32,
    pub dry_run: bool,
    pub simulation: bool,
    pub session_id: Option<String>,
}

impl Default for HarnessConfig {
    fn default() -> Self {
        Self {
            checkpoint_interval: 10,
            max_interventions: 20,
            dry_run: false,
            simulation: false,
            session_id: None,
        }
    }
}

pub struct HarnessBuilder {
    config: HarnessConfig,
    observers: Vec<Arc<dyn Observer>>,
    detectors: Vec<Arc<dyn Detector>>,
    strategies: Vec<Arc<dyn Strategy>>,
    audit_store: Option<Arc<dyn AuditStore>>,
}

impl HarnessBuilder {
    pub fn new() -> Self {
        Self {
            config: HarnessConfig::default(),
            observers: Vec::new(),
            detectors: Vec::new(),
            strategies: Vec::new(),
            audit_store: None,
        }
    }

    pub fn config(mut self, config: HarnessConfig) -> Self {
        self.config = config;
        self
    }

    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.config.dry_run = dry_run;
        self
    }

    pub fn simulation(mut self, sim: bool) -> Self {
        self.config.simulation = sim;
        self
    }

    pub fn observe(mut self, observers: Vec<Arc<dyn Observer>>) -> Self {
        self.observers = observers;
        self
    }

    pub fn detect(mut self, detectors: Vec<Arc<dyn Detector>>) -> Self {
        self.detectors = detectors;
        self
    }

    pub fn strategize(mut self, strategies: Vec<Arc<dyn Strategy>>) -> Self {
        self.strategies = strategies;
        self
    }

    pub fn audit(mut self, store: Arc<dyn AuditStore>) -> Self {
        self.audit_store = Some(store);
        self
    }

    pub fn build(self) -> Harness {
        Harness {
            config: self.config,
            observers: self.observers,
            detectors: self.detectors,
            strategies: self.strategies,
            audit_store: self.audit_store,
        }
    }
}

impl Default for HarnessBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Harness {
    pub config: HarnessConfig,
    pub observers: Vec<Arc<dyn Observer>>,
    pub detectors: Vec<Arc<dyn Detector>>,
    pub strategies: Vec<Arc<dyn Strategy>>,
    pub audit_store: Option<Arc<dyn AuditStore>>,
}

impl Harness {
    pub fn builder() -> HarnessBuilder {
        HarnessBuilder::new()
    }

    pub async fn run(
        &self,
        _agent: &mut dyn AgentAdapter,
        _task: &str,
    ) -> Result<HarnessRunResult, ForgeError> {
        // Full implementation in forge-harness runtime
        Ok(HarnessRunResult::default())
    }
}

#[derive(Debug, Clone, Default)]
pub struct HarnessRunResult {
    pub agent_id: String,
    pub observation_count: u64,
    pub detection_count: u64,
    pub intervention_count: u64,
    pub success: bool,
}
