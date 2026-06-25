use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub tags: Vec<String>,
    pub preset: Option<String>,
    pub observers: Vec<String>,
    pub detectors: Vec<String>,
    pub strategies: Vec<String>,
}

pub struct SkillRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillRegistry {
    pub fn new() -> Self { Self { skills: HashMap::new() } }
    pub fn register(&mut self, skill: Skill) { self.skills.insert(skill.name.clone(), skill); }
    pub fn get(&self, name: &str) -> Option<&Skill> { self.skills.get(name) }
    pub fn list(&self) -> Vec<&Skill> { self.skills.values().collect() }
    pub fn count(&self) -> usize { self.skills.len() }

    pub fn builtin() -> Self {
        let mut registry = Self::new();
        registry.register(Skill {
            name: "security-first".into(), version: "1.0.0".into(),
            description: "Maximum security posture".into(), author: "forgelabs".into(),
            tags: vec!["security".into(), "production".into()], preset: Some("solo".into()),
            observers: vec!["security".into(), "compliance".into()],
            detectors: vec!["secret_leak".into(), "prompt_injection".into()],
            strategies: vec!["circuit_break".into(), "isolate".into(), "quarantine".into()],
        });
        registry.register(Skill {
            name: "cost-optimizer".into(), version: "1.0.0".into(),
            description: "Minimum cost configuration".into(), author: "forgelabs".into(),
            tags: vec!["cost".into(), "dev".into()], preset: Some("solo".into()),
            observers: vec!["token".into(), "cost".into()],
            detectors: vec!["cost_anomaly".into(), "model_mismatch".into()],
            strategies: vec!["degrade".into(), "escalate".into()],
        });
        registry.register(Skill {
            name: "accuracy-max".into(), version: "1.0.0".into(),
            description: "Maximum accuracy with adversarial verification".into(), author: "forgelabs".into(),
            tags: vec!["quality".into()], preset: Some("solo".into()),
            observers: vec!["accuracy".into(), "context_quality".into()],
            detectors: vec!["accuracy_risk".into(), "hallucination".into(), "variety_collapse".into()],
            strategies: vec!["diversify".into(), "nudge".into(), "replace".into()],
        });
        registry
    }
}
