use std::collections::HashMap;
use forge_sdk::error::ForgeError;
use crate::SessionAudit;

pub struct WeaknessMiner { min_sessions: usize }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WeaknessPattern {
    pub id: String,
    pub signature: FailureSignature,
    pub model_family: String,
    pub agent_type: String,
    pub occurrence_count: u32,
    pub severity_score: f64,
    pub evidence_sessions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum FailureSignature {
    LateDetection { detector: String, avg_detection_turn: f64, consequence: String },
    MissedDetection { pattern_description: String, should_have_detected: String },
    IneffectiveIntervention { strategy: String, reason: String },
    FalsePositive { detector: String, false_positive_rate: f64 },
    ModelSpecific { model: String, pattern: String },
}

impl WeaknessMiner {
    pub fn new(min_sessions: usize) -> Self { Self { min_sessions } }

    pub fn mine(&self, audits: &[SessionAudit]) -> Result<Vec<WeaknessPattern>, ForgeError> {
        if audits.len() < self.min_sessions {
            return Ok(Vec::new());
        }

        let mut patterns: Vec<WeaknessPattern> = Vec::new();

        // ─── 1. Find late detection patterns ───
        let mut late_detections: HashMap<String, (u32, f64, String)> = HashMap::new();
        for audit in audits {
            if !audit.success && audit.detection_count > 0 {
                let key = format!("late_detection_{}", audit.agent_type);
                let entry = late_detections.entry(key).or_insert((0, 0.0, "context_overflow".into()));
                entry.0 += 1;
                entry.1 += audit.duration_secs;
            }
        }
        for (_key, (count, avg_time, consequence)) in &late_detections {
            if *count >= 3 {
                patterns.push(WeaknessPattern {
                    id: format!("pat_{}", patterns.len()),
                    signature: FailureSignature::LateDetection {
                        detector: "loop".into(),
                        avg_detection_turn: *avg_time / *count as f64,
                        consequence: consequence.clone(),
                    },
                    model_family: "all".into(), agent_type: "solo".into(),
                    occurrence_count: *count,
                    severity_score: (*count as f64 / audits.len() as f64).min(1.0),
                    evidence_sessions: audits.iter().filter(|a| !a.success).map(|a| a.session_id.clone()).collect(),
                });
            }
        }

        // ─── 2. Find ineffective intervention patterns ───
        let mut ineffective: HashMap<String, u32> = HashMap::new();
        for audit in audits {
            if audit.intervention_count > 0 && !audit.success {
                for pattern in &audit.failure_patterns {
                    *ineffective.entry(pattern.clone()).or_default() += 1;
                }
            }
        }
        for (pattern, count) in &ineffective {
            if *count >= 3 {
                patterns.push(WeaknessPattern {
                    id: format!("pat_{}", patterns.len()),
                    signature: FailureSignature::IneffectiveIntervention {
                        strategy: "nudge".into(),
                        reason: pattern.clone(),
                    },
                    model_family: "all".into(), agent_type: "solo".into(),
                    occurrence_count: *count,
                    severity_score: (*count as f64 / audits.len() as f64 * 2.0).min(1.0),
                    evidence_sessions: Vec::new(),
                });
            }
        }

        // ─── 3. Find model-specific failure patterns ───
        let mut model_failures: HashMap<String, (u32, f64)> = HashMap::new();
        for audit in audits {
            let _key = format!("{}_{}", audit.model, if audit.success { "success" } else { "failure" });
            let entry = model_failures.entry(audit.model.clone()).or_insert((0, 0.0));
            if !audit.success { entry.0 += 1; }
        }
        for (model, (failures, _)) in &model_failures {
            let total = audits.iter().filter(|a| &a.model == model).count();
            let fail_rate = *failures as f64 / total.max(1) as f64;
            if fail_rate > 0.3 && *failures >= 3 {
                patterns.push(WeaknessPattern {
                    id: format!("pat_{}", patterns.len()),
                    signature: FailureSignature::ModelSpecific {
                        model: model.clone(),
                        pattern: format!("{:.0}% failure rate", fail_rate * 100.0),
                    },
                    model_family: model.clone(), agent_type: "all".into(),
                    occurrence_count: *failures,
                    severity_score: fail_rate,
                    evidence_sessions: audits.iter().filter(|a| &a.model == model && !a.success).map(|a| a.session_id.clone()).collect(),
                });
            }
        }

        // ─── 4. Cluster similar failure patterns ───
        let mut clustered: HashMap<String, Vec<&SessionAudit>> = HashMap::new();
        for audit in audits {
            if !audit.success {
                for msg in &audit.error_messages {
                    let key = msg.chars().take(40).collect::<String>();
                    clustered.entry(key).or_default().push(audit);
                }
            }
        }
        for (cluster_key, cluster_audits) in &clustered {
            if cluster_audits.len() >= 3 {
                patterns.push(WeaknessPattern {
                    id: format!("pat_{}", patterns.len()),
                    signature: FailureSignature::MissedDetection {
                        pattern_description: cluster_key.clone(),
                        should_have_detected: "error_pattern".into(),
                    },
                    model_family: "all".into(), agent_type: "all".into(),
                    occurrence_count: cluster_audits.len() as u32,
                    severity_score: (cluster_audits.len() as f64 / 10.0).min(1.0),
                    evidence_sessions: cluster_audits.iter().map(|a| a.session_id.clone()).collect(),
                });
            }
        }

        // Sort by severity
        patterns.sort_by(|a, b| b.severity_score.partial_cmp(&a.severity_score).unwrap());
        Ok(patterns)
    }
}
