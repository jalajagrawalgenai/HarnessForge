// forge-harness/src/human_gate.rs — Human-in-the-loop gates

use forge_sdk::types::detection::Severity;

#[derive(Debug, Clone)]
pub struct HumanGateConfig {
    pub before_dangerous_tools: bool,
    pub on_cost_spike_multiplier: f64,
    pub on_accuracy_drop: f64,
    pub on_severity: Severity,
    pub auto_resume_timeout_secs: u64,
}

impl Default for HumanGateConfig {
    fn default() -> Self {
        Self {
            before_dangerous_tools: true,
            on_cost_spike_multiplier: 5.0,
            on_accuracy_drop: 0.7,
            on_severity: Severity::Critical,
            auto_resume_timeout_secs: 1800, // 30 minutes
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateState {
    Open,     // Agent can proceed
    Paused,   // Waiting for human
    Approved, // Human approved
    Rejected, // Human rejected
    Override, // Human overrode harness decision
}

#[derive(Debug)]
pub struct HumanGate {
    config: HumanGateConfig,
    state: GateState,
    reason: Option<String>,
}

impl HumanGate {
    pub fn new(config: HumanGateConfig) -> Self {
        Self {
            config,
            state: GateState::Open,
            reason: None,
        }
    }

    /// Check if agent should be paused based on current conditions
    pub fn should_pause(&self, severity: &Severity, cost_multiplier: f64, accuracy: f64) -> bool {
        if *severity == Severity::Critical && self.config.on_severity == Severity::Critical {
            return true;
        }
        if cost_multiplier > self.config.on_cost_spike_multiplier {
            return true;
        }
        if accuracy < self.config.on_accuracy_drop {
            return true;
        }
        false
    }

    pub fn pause(&mut self, reason: String) {
        self.state = GateState::Paused;
        self.reason = Some(reason);
    }

    pub fn approve(&mut self) {
        self.state = GateState::Approved;
    }

    pub fn reject(&mut self) {
        self.state = GateState::Rejected;
    }

    pub fn override_gate(&mut self) {
        self.state = GateState::Override;
    }

    pub fn state(&self) -> &GateState {
        &self.state
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}
