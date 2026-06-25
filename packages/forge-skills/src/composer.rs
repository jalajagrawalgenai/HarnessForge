use crate::registry::{Skill, SkillRegistry};

pub struct SkillComposer;

impl SkillComposer {
    pub fn compose(skills: &[&Skill]) -> ComposedConfig {
        let mut observers = Vec::new();
        let mut detectors = Vec::new();
        let mut strategies = Vec::new();
        let mut preset = None;
        for skill in skills {
            for o in &skill.observers { if !observers.contains(o) { observers.push(o.clone()); } }
            for d in &skill.detectors { if !detectors.contains(d) { detectors.push(d.clone()); } }
            for s in &skill.strategies { if !strategies.contains(s) { strategies.push(s.clone()); } }
            if skill.preset.is_some() { preset = skill.preset.clone(); }
        }
        ComposedConfig { preset, observers, detectors, strategies }
    }
}

#[derive(Debug, Clone)]
pub struct ComposedConfig {
    pub preset: Option<String>,
    pub observers: Vec<String>,
    pub detectors: Vec<String>,
    pub strategies: Vec<String>,
}
