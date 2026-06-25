use async_trait::async_trait;
use std::path::Path;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct HallucinationDetector { project_root: String }

impl HallucinationDetector { pub fn new(root: &str) -> Self { Self { project_root: root.into() } } }

#[async_trait]
impl Detector for HallucinationDetector {
    fn name(&self) -> &'static str { "hallucination" }
    fn description(&self) -> &'static str { "Detects references to non-existent files/APIs" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        obs.iter().filter_map(|o| {
            let file_ref = o.get("file_reference").and_then(|v| v.as_str())?;
            let full = Path::new(&self.project_root).join(file_ref);
            if !full.exists() {
                Some(DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                    severity: Severity::Warning,
                    category: IssueCategory::Hallucination { reference: file_ref.into(), reference_type: "file".into() },
                    description: format!("Agent referenced non-existent file: {}", file_ref),
                    confidence: 1.0, suggested_actions: vec!["nudge".into()],
                    evidence_summary: format!("File '{}' not found", file_ref) })
            } else { None }
        }).collect()
    }
}
