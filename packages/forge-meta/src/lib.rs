pub mod ab_testing;
pub mod confidence;
pub mod cross_model;
pub mod edit_registry;
pub mod efficacy;
pub mod harness_proposer;
pub mod manual_edit;
pub mod notifications;
pub mod proposal_validator;
pub mod scheduler;
pub mod weakness_miner;

use std::sync::Arc;
use forge_sdk::error::ForgeError;
use crate::weakness_miner::{WeaknessMiner, WeaknessPattern};
use crate::harness_proposer::{HarnessProposer, HarnessEdit};
use crate::proposal_validator::{ProposalValidator, ValidatedEdit};
use crate::edit_registry::EditRegistry;
use crate::ab_testing::ABTestEngine;

/// The self-improving meta-harness. Runs the Self-Harness loop:
/// Mine weaknesses → Propose edits → Validate → Apply
pub struct MetaHarness {
    miner: WeaknessMiner,
    proposer: HarnessProposer,
    validator: ProposalValidator,
    registry: EditRegistry,
    ab_engine: ABTestEngine,
}

#[derive(Debug, Clone)]
pub struct MetaConfig {
    pub min_sessions: usize,
    pub max_edits_per_cycle: usize,
    pub regression_test_count: usize,
    pub min_improvement_pct: f64,
    pub max_regressions: usize,
    pub significance_level: f64,
}

impl Default for MetaConfig {
    fn default() -> Self {
        Self {
            min_sessions: 50,
            max_edits_per_cycle: 10,
            regression_test_count: 25,
            min_improvement_pct: 3.0,
            max_regressions: 0,
            significance_level: 0.05,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ImprovementResult {
    pub patterns_found: usize,
    pub edits_proposed: usize,
    pub edits_accepted: usize,
    pub edits_rejected: usize,
    pub avg_improvement_pct: f64,
    pub new_harness_version: String,
}

impl MetaHarness {
    pub fn new(config: MetaConfig) -> Self {
        Self {
            miner: WeaknessMiner::new(config.min_sessions),
            proposer: HarnessProposer::new(),
            validator: ProposalValidator::new(
                config.regression_test_count,
                config.min_improvement_pct,
                config.max_regressions,
                config.significance_level,
            ),
            registry: EditRegistry::new(),
            ab_engine: ABTestEngine::new(),
        }
    }

    /// Run one complete improvement cycle
    pub async fn improve(
        &mut self,
        session_audits: &[SessionAudit],
        current_harness: &serde_json::Value,
        held_out_tasks: &[String],
    ) -> Result<ImprovementResult, ForgeError> {
        // 1. Mine weaknesses
        let patterns = self.miner.mine(session_audits)?;
        if patterns.is_empty() {
            return Ok(ImprovementResult {
                patterns_found: 0, edits_proposed: 0, edits_accepted: 0,
                edits_rejected: 0, avg_improvement_pct: 0.0,
                new_harness_version: self.registry.current_version(),
            });
        }

        // 2. Propose edits
        let edits = self.proposer.propose(&patterns, current_harness)?;

        // 3. Validate edits via regression testing
        let validated = self.validator.validate(&edits, held_out_tasks).await?;

        // 4. Register accepted edits
        let mut accepted = 0;
        let mut rejected = 0;
        let mut total_improvement = 0.0;

        for v in &validated {
            if v.accepted {
                self.registry.apply(v.edit.clone())?;
                accepted += 1;
                total_improvement += v.pass_rate_delta;
            } else {
                rejected += 1;
            }
        }

        let new_version = self.registry.current_version();

        Ok(ImprovementResult {
            patterns_found: patterns.len(),
            edits_proposed: edits.len(),
            edits_accepted: accepted,
            edits_rejected: rejected,
            avg_improvement_pct: if accepted > 0 { total_improvement / accepted as f64 } else { 0.0 },
            new_harness_version: new_version,
        })
    }

    pub fn registry(&self) -> &EditRegistry { &self.registry }
}

/// Represents a completed session's audit data for mining
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionAudit {
    pub session_id: String,
    pub agent_type: String,
    pub model: String,
    pub success: bool,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub duration_secs: f64,
    pub detection_count: u64,
    pub intervention_count: u64,
    pub failure_patterns: Vec<String>,
    pub error_messages: Vec<String>,
}
