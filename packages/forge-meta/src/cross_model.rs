use crate::harness_proposer::HarnessEdit;
use crate::weakness_miner::WeaknessPattern;

pub struct CrossModelLearner;

impl CrossModelLearner {
    pub fn transfer(
        pattern: &WeaknessPattern,
        target_model: &str,
        existing_patterns: &[WeaknessPattern],
    ) -> Option<HarnessEdit> {
        let already_has = existing_patterns
            .iter()
            .any(|p| p.model_family == target_model);
        if already_has {
            return None;
        }
        Some(HarnessEdit {
            id: format!("xfer_{}", uuid::Uuid::new_v4()),
            weakness_id: pattern.id.clone(),
            target: crate::harness_proposer::EditTarget::PresetUpdate {
                preset: format!("model_{}", target_model),
            },
            change: crate::harness_proposer::EditChange::ModifyValue {
                old: serde_json::json!("default"),
                new: serde_json::json!("cross_model_optimized"),
            },
            rationale: format!(
                "Cross-model transfer: pattern from {} applied to {}",
                pattern.model_family, target_model
            ),
            affected_models: vec![target_model.into()],
            min_chars: 60,
        })
    }
}
