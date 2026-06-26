use axum::extract::Path;
use axum::Json;
use forge_skills::composer::SkillComposer;
use forge_skills::registry::SkillRegistry;
use forge_skills::sdk::SkillSdk;
use serde_json::{json, Value};

pub async fn list() -> Json<Value> {
    let registry = SkillRegistry::builtin();
    let skills: Vec<Value> = registry.list().iter().map(|s| {
        json!({"name":s.name,"version":s.version,"description":s.description,"author":s.author,"tags":s.tags,"preset":s.preset,"observers":s.observers,"detectors":s.detectors,"strategies":s.strategies})
    }).collect();
    Json(json!({"skills": skills, "total": skills.len()}))
}

pub async fn get(Path(name): Path<String>) -> Json<Value> {
    let registry = SkillRegistry::builtin();
    match registry.get(&name) {
        Some(s) => Json(
            json!({"name":s.name,"version":s.version,"description":s.description,"author":s.author,"tags":s.tags,"observers":s.observers,"detectors":s.detectors,"strategies":s.strategies}),
        ),
        None => Json(json!({"error":"skill not found","name":name})),
    }
}

pub async fn compose(Json(body): Json<Value>) -> Json<Value> {
    let skill_names: Vec<&str> = body
        .get("skills")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let registry = SkillRegistry::builtin();
    let skills: Vec<_> = skill_names
        .iter()
        .filter_map(|name| registry.get(name))
        .collect();
    if skills.is_empty() {
        return Json(json!({"error":"no valid skills found"}));
    }
    let skill_refs = skills.to_vec();
    let composed = SkillComposer::compose(&skill_refs);
    Json(
        json!({"preset":composed.preset,"observers":composed.observers,"detectors":composed.detectors,"strategies":composed.strategies}),
    )
}

pub async fn validate(Json(body): Json<Value>) -> Json<Value> {
    let skill = forge_skills::registry::Skill {
        name: body
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed")
            .into(),
        version: body
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.1.0")
            .into(),
        description: body
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .into(),
        author: body
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .into(),
        tags: body
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        preset: body
            .get("preset")
            .and_then(|v| v.as_str())
            .map(String::from),
        observers: body
            .get("observers")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        detectors: body
            .get("detectors")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        strategies: body
            .get("strategies")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
    };
    let errors = SkillSdk::validate(&skill);
    Json(json!({"valid": errors.is_empty(), "errors": errors}))
}

pub async fn scaffold(Json(body): Json<Value>) -> Json<Value> {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("my-skill");
    let skill = SkillSdk::scaffold(name);
    Json(
        json!({"name":skill.name,"version":skill.version,"description":skill.description,"observers":skill.observers,"detectors":skill.detectors,"strategies":skill.strategies,"tags":skill.tags}),
    )
}
