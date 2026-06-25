use crate::weakness_miner::WeaknessPattern;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizedPattern {
    pub signature_hash: String, pub model_family: String, pub agent_type: String,
    pub occurrence_count: u32, pub severity_score: f64, pub suggested_fix: Option<String>,
}

pub struct FederatedRegistry { patterns: Vec<AnonymizedPattern> }

impl FederatedRegistry {
    pub fn new() -> Self { Self { patterns: Vec::new() } }
    pub fn share(&mut self, pattern: &WeaknessPattern) -> AnonymizedPattern {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        format!("{:?}", pattern.signature).hash(&mut h);
        let ap = AnonymizedPattern {
            signature_hash: format!("{:x}", h.finish()), model_family: pattern.model_family.clone(),
            agent_type: pattern.agent_type.clone(), occurrence_count: pattern.occurrence_count,
            severity_score: pattern.severity_score, suggested_fix: None,
        };
        self.patterns.push(ap.clone());
        ap
    }
    pub fn find_similar(&self, pattern: &WeaknessPattern) -> Vec<&AnonymizedPattern> {
        self.patterns.iter().filter(|p| p.model_family == pattern.model_family && p.agent_type == pattern.agent_type).collect()
    }
    pub fn count(&self) -> usize { self.patterns.len() }
}
