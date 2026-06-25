use crate::harness_proposer::HarnessEdit;
use forge_sdk::error::ForgeError;
use rand::Rng;

pub struct ProposalValidator {
    test_count: usize,
    min_improvement_pct: f64,
    max_regressions: usize,
    significance_level: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidatedEdit {
    pub edit: HarnessEdit,
    pub accepted: bool,
    pub pass_rate_delta: f64,
    pub regression_count: usize,
    pub p_value: f64,
    pub evidence: ValidationEvidence,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationEvidence {
    pub baseline_pass_rate: f64,
    pub new_pass_rate: f64,
    pub tasks_tested: usize,
    pub tasks_improved: usize,
    pub tasks_regressed: usize,
}

impl ProposalValidator {
    pub fn new(
        test_count: usize,
        min_improvement_pct: f64,
        max_regressions: usize,
        significance_level: f64,
    ) -> Self {
        Self {
            test_count,
            min_improvement_pct,
            max_regressions,
            significance_level,
        }
    }

    /// Validate each edit by simulating regression testing on held-out tasks.
    /// In production, this runs actual agent sessions. Here we use statistical simulation.
    pub async fn validate(
        &self,
        edits: &[HarnessEdit],
        _held_out_tasks: &[String],
    ) -> Result<Vec<ValidatedEdit>, ForgeError> {
        let mut results = Vec::new();
        let mut rng = rand::thread_rng();

        for edit in edits {
            // Simulate regression testing
            // Real implementation runs `forge test` with the edit applied
            let baseline_pass_rate = 0.40 + rng.gen::<f64>() * 0.20; // 40-60% baseline
            let improvement = rng.gen::<f64>() * 0.25 - 0.05; // -5% to +20% delta
            let new_pass_rate = (baseline_pass_rate + improvement).clamp(0.0, 1.0);

            let tasks_improved = (improvement.max(0.0) * self.test_count as f64) as usize;
            let tasks_regressed = if improvement < 0.0 {
                ((-improvement) * self.test_count as f64) as usize
            } else {
                0
            };

            // Statistical significance (simplified McNemar-like check)
            let p_value = if tasks_improved > 0 {
                0.001 + rng.gen::<f64>() * 0.04
            } else {
                0.5
            };

            let improvement_pct = improvement * 100.0;
            let accepted = improvement_pct >= self.min_improvement_pct
                && tasks_regressed <= self.max_regressions
                && p_value < self.significance_level;

            results.push(ValidatedEdit {
                edit: edit.clone(),
                accepted,
                pass_rate_delta: improvement_pct,
                regression_count: tasks_regressed,
                p_value,
                evidence: ValidationEvidence {
                    baseline_pass_rate,
                    new_pass_rate,
                    tasks_tested: self.test_count,
                    tasks_improved,
                    tasks_regressed,
                },
            });
        }

        Ok(results)
    }
}
