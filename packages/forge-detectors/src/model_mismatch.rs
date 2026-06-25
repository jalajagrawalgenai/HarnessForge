use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct ModelMismatchDetector;
const COMPLEX_KEYWORDS: &[&str] = &["refactor","architecture","security","migrate","implement","design","optimize"];

#[async_trait]
impl Detector for ModelMismatchDetector {
    fn name(&self) -> &'static str { "model_mismatch" }
    fn description(&self) -> &'static str { "Detects wrong model for task complexity" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        for o in obs {
            if let (Some(task), Some(model)) = (o.get("task").and_then(|v| v.as_str()), o.get("model").and_then(|v| v.as_str())) {
                let is_complex = COMPLEX_KEYWORDS.iter().any(|k| task.to_lowercase().contains(&k.to_lowercase()));
                let is_cheap_model = model.contains("haiku") || model.contains("flash") || model.contains("mini");
                if is_complex && is_cheap_model {
                    return vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                        severity: Severity::Warning,
                        category: IssueCategory::ModelMismatch {
                            task_complexity: "complex".into(), model_used: model.into(),
                            suggested_model: "claude-sonnet-4-6".into(),
                        },
                        description: format!("Complex task '{}' assigned to cheap model {}", task, model),
                        confidence: 0.85,
                        suggested_actions: vec!["escalate".into()],
                        evidence_summary: format!("Task complexity mismatch: {} on {}", task, model) }];
                }
            }
        }
        vec![]
    }
}
