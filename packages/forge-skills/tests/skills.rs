use forge_skills::composer::SkillComposer;
use forge_skills::registry::SkillRegistry;
use forge_skills::sdk::SkillSdk;

#[test]
fn test_builtin_skills() {
    let registry = SkillRegistry::builtin();
    assert_eq!(registry.count(), 3);
    assert!(registry.get("security-first").is_some());
    assert!(registry.get("cost-optimizer").is_some());
    assert!(registry.get("accuracy-max").is_some());
}

#[test]
fn test_skill_composer() {
    let registry = SkillRegistry::builtin();
    let sec = registry.get("security-first").unwrap();
    let cost = registry.get("cost-optimizer").unwrap();
    let composed = SkillComposer::compose(&[sec, cost]);
    assert!(!composed.observers.is_empty());
    assert!(!composed.detectors.is_empty());
    assert!(!composed.strategies.is_empty());
}

#[test]
fn test_skill_validate() {
    let skill = SkillSdk::scaffold("test-skill");
    let errors = SkillSdk::validate(&skill);
    assert!(!errors.is_empty()); // Missing description
}
