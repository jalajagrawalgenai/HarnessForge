use crate::harness_proposer::HarnessEdit;
use crate::proposal_validator::{ProposalValidator, ValidatedEdit};
use forge_sdk::error::ForgeError;

pub struct ManualEditEngine { validator: ProposalValidator }

impl ManualEditEngine {
    pub fn new(validator: ProposalValidator) -> Self { Self { validator } }
    pub async fn validate_manual_edit(&self, edit: &HarnessEdit, tasks: &[String]) -> Result<ValidatedEdit, ForgeError> {
        let results = self.validator.validate(std::slice::from_ref(edit), tasks).await?;
        Ok(results.into_iter().next().unwrap_or(ValidatedEdit { edit: edit.clone(), accepted: false, pass_rate_delta: 0.0, regression_count: 0, p_value: 1.0, evidence: crate::proposal_validator::ValidationEvidence { baseline_pass_rate: 0.0, new_pass_rate: 0.0, tasks_tested: 0, tasks_improved: 0, tasks_regressed: 0 } }))
    }
}
