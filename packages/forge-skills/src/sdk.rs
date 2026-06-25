use crate::registry::Skill;

pub struct SkillSdk;

impl SkillSdk {
    pub fn scaffold(name: &str) -> Skill {
        Skill {
            name: name.into(), version: "0.1.0".into(),
            description: "TODO".into(), author: "TODO".into(),
            tags: vec![], preset: None,
            observers: vec![], detectors: vec![], strategies: vec![],
        }
    }

    pub fn validate(skill: &Skill) -> Vec<String> {
        let mut errors = Vec::new();
        if skill.observers.is_empty() && skill.detectors.is_empty() {
            errors.push("Skill must specify at least one observer or detector".into());
        }
        if skill.description == "TODO" { errors.push("Description is required".into()); }
        errors
    }
}
