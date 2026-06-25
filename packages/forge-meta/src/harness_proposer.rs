use forge_sdk::error::ForgeError;
use crate::weakness_miner::{WeaknessPattern, FailureSignature};

pub struct HarnessProposer;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HarnessEdit {
    pub id: String,
    pub weakness_id: String,
    pub target: EditTarget,
    pub change: EditChange,
    pub rationale: String,
    pub affected_models: Vec<String>,
    pub min_chars: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EditTarget {
    DetectorThreshold { detector: String, parameter: String },
    StrategyParameter { strategy: String, parameter: String },
    AddDetectorRule { detector: String },
    PresetUpdate { preset: String },
    ObserverThreshold { observer: String, parameter: String },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum EditChange {
    ModifyValue { old: serde_json::Value, new: serde_json::Value },
    AddRule { condition: String, action: String },
    RemoveRule { rule_id: String },
}

impl HarnessProposer {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self }

    pub fn propose(
        &self,
        patterns: &[WeaknessPattern],
        _current_harness: &serde_json::Value,
    ) -> Result<Vec<HarnessEdit>, ForgeError> {
        let mut edits = Vec::new();

        for pattern in patterns {
            let candidates = self.generate_candidates(pattern);
            edits.extend(candidates);
        }

        // Deduplicate and limit
        edits.sort_by_key(|b| std::cmp::Reverse(b.rationale.len()));
        edits.dedup_by(|a, b| a.target_matches(b));

        Ok(edits)
    }

    fn generate_candidates(&self, pattern: &WeaknessPattern) -> Vec<HarnessEdit> {
        let mut candidates = Vec::new();

        match &pattern.signature {
            FailureSignature::LateDetection { detector, avg_detection_turn, .. } => {
                // Lower the detection threshold for this detector
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::DetectorThreshold {
                        detector: detector.clone(),
                        parameter: "threshold".into(),
                    },
                    change: EditChange::ModifyValue {
                        old: serde_json::json!(6),
                        new: serde_json::json!(4),
                    },
                    rationale: format!(
                        "Lower {} detection threshold: avg detection at turn {:.0}, too late to prevent failure. Reducing threshold catches this earlier.",
                        detector, avg_detection_turn
                    ),
                    affected_models: vec![pattern.model_family.clone()],
                    min_chars: 80,
                });
                // Also compact earlier
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::StrategyParameter {
                        strategy: "compact".into(),
                        parameter: "trigger_pressure".into(),
                    },
                    change: EditChange::ModifyValue {
                        old: serde_json::json!(0.85),
                        new: serde_json::json!(0.75),
                    },
                    rationale: "Compact context earlier to prevent overflow before loop detection fires.".into(),
                    affected_models: vec![pattern.model_family.clone()],
                    min_chars: 60,
                });
            }

            FailureSignature::IneffectiveIntervention { strategy, reason } => {
                // Escalate strategy priority or switch strategy
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::StrategyParameter {
                        strategy: strategy.clone(),
                        parameter: "max_applications".into(),
                    },
                    change: EditChange::ModifyValue {
                        old: serde_json::json!(3),
                        new: serde_json::json!(2),
                    },
                    rationale: format!(
                        "Strategy '{}' was ineffective for: {}. Reducing max applications before escalating to stronger intervention.",
                        strategy, reason
                    ),
                    affected_models: vec![pattern.model_family.clone()],
                    min_chars: 70,
                });
            }

            FailureSignature::MissedDetection { pattern_description, should_have_detected } => {
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::AddDetectorRule { detector: should_have_detected.clone() },
                    change: EditChange::AddRule {
                        condition: pattern_description.clone(),
                        action: format!("trigger_{}", should_have_detected),
                    },
                    rationale: format!(
                        "Adding new detection rule for '{}' — currently undetected failure pattern.",
                        pattern_description
                    ),
                    affected_models: vec![pattern.model_family.clone()],
                    min_chars: 50,
                });
            }

            FailureSignature::ModelSpecific { model, pattern: failure_pattern } => {
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::PresetUpdate { preset: format!("model_{}", model) },
                    change: EditChange::ModifyValue {
                        old: serde_json::json!("default"),
                        new: serde_json::json!(format!("optimized_for_{}", model)),
                    },
                    rationale: format!(
                        "Model {} has a {}. Creating model-specific preset to address this.",
                        model, failure_pattern
                    ),
                    affected_models: vec![model.clone()],
                    min_chars: 60,
                });
            }

            FailureSignature::FalsePositive { detector, false_positive_rate: fpr } => {
                candidates.push(HarnessEdit {
                    id: format!("edit_{}", uuid::Uuid::new_v4()),
                    weakness_id: pattern.id.clone(),
                    target: EditTarget::DetectorThreshold {
                        detector: detector.clone(),
                        parameter: "threshold".into(),
                    },
                    change: EditChange::ModifyValue {
                        old: serde_json::json!(3),
                        new: serde_json::json!(5),
                    },
                    rationale: format!(
                        "{} has {:.0}% false positive rate. Raising threshold to reduce noise.",
                        detector, fpr * 100.0
                    ),
                    affected_models: vec![pattern.model_family.clone()],
                    min_chars: 50,
                });
            }
        }

        candidates
    }
}

impl HarnessEdit {
    fn target_matches(&self, other: &HarnessEdit) -> bool {
        std::mem::discriminant(&self.target) == std::mem::discriminant(&other.target)
    }
}
