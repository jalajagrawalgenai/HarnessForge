use crate::weakness_miner::WeaknessPattern;

pub struct ConfidenceScorer;

impl ConfidenceScorer {
    pub fn score(pattern: &WeaknessPattern) -> f64 {
        let frequency_score = (pattern.occurrence_count as f64 / 20.0).min(1.0);
        let evidence_score = (pattern.evidence_sessions.len() as f64 / 10.0).min(1.0);
        let severity_component = pattern.severity_score;
        (frequency_score * 0.3 + evidence_score * 0.3 + severity_component * 0.4).clamp(0.0, 1.0)
    }
}
