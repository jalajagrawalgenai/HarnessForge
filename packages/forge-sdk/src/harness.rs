use async_trait::async_trait;
use std::sync::Arc;

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

// ─── HarnessBuilder ───────────────────────────────────────────────────

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

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.config.session_id = Some(id.into());
        self
    }

    pub fn observe(mut self, observers: Vec<Arc<dyn Observer>>) -> Self {
        self.observers = observers;
        self
    }

    pub fn add_observer(mut self, observer: Arc<dyn Observer>) -> Self {
        self.observers.push(observer);
        self
    }

    pub fn detect(mut self, detectors: Vec<Arc<dyn Detector>>) -> Self {
        self.detectors = detectors;
        self
    }

    pub fn add_detector(mut self, detector: Arc<dyn Detector>) -> Self {
        self.detectors.push(detector);
        self
    }

    pub fn strategize(mut self, strategies: Vec<Arc<dyn Strategy>>) -> Self {
        self.strategies = strategies;
        self
    }

    pub fn add_strategy(mut self, strategy: Arc<dyn Strategy>) -> Self {
        self.strategies.push(strategy);
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

// ─── Harness ──────────────────────────────────────────────────────────

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

    /// Run the harness against an agent.
    ///
    /// This default implementation runs the agent through the channels
    /// and returns a basic result. For the full observe→detect→strategy
    /// →action→audit pipeline, use:
    /// - `Harness::run_with()` with a `HarnessRuntime` implementation
    /// - `forge_harness::runner::run_harness_session()` directly
    pub async fn run(
        &self,
        agent: &mut dyn AgentAdapter,
        task: &str,
    ) -> Result<HarnessRunResult, ForgeError> {
        let agent_id = agent.id();
        let (event_tx, _event_rx) = tokio::sync::mpsc::channel::<crate::events::AgentEvent>(256);
        let (_intervention_tx, intervention_rx) =
            tokio::sync::mpsc::channel::<crate::events::Intervention>(64);

        let outcome = agent.run(task, event_tx, intervention_rx).await?;

        Ok(HarnessRunResult {
            agent_id,
            observation_count: self.observers.len() as u64,
            detection_count: self.detectors.len() as u64,
            intervention_count: self.strategies.len() as u64,
            success: outcome.success,
        })
    }
}

// ─── HarnessRuntime trait ─────────────────────────────────────────────

#[async_trait]
pub trait HarnessRuntime: Send + Sync {
    /// Execute a full harness session: spawn agent, run pipeline, return result.
    async fn execute(
        &self,
        harness: &Harness,
        agent: &mut (dyn AgentAdapter + Send),
        task: &str,
    ) -> Result<HarnessRunResult, ForgeError>;
}

impl Harness {
    pub async fn run_with(
        &self,
        runtime: &dyn HarnessRuntime,
        agent: &mut (dyn AgentAdapter + Send),
        task: &str,
    ) -> Result<HarnessRunResult, ForgeError> {
        runtime.execute(self, agent, task).await
    }
}

// ─── HarnessRunResult ─────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct HarnessRunResult {
    pub agent_id: String,
    pub observation_count: u64,
    pub detection_count: u64,
    pub intervention_count: u64,
    pub success: bool,
}
