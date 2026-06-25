// LoopDetector — detects when agent calls same tool repeatedly with no progress

use async_trait::async_trait;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct LoopDetector {
    /// (tool_name, args_hash) → count
    #[allow(dead_code)]
    tool_frequency: Mutex<HashMap<String, u32>>,
    threshold: u32,
    window_turns: u32,
}

impl LoopDetector {
    pub fn new(threshold: u32, window_turns: u32) -> Self {
        Self {
            tool_frequency: Mutex::new(HashMap::new()),
            threshold,
            window_turns,
        }
    }

    #[allow(dead_code)]
    fn hash_args(args: &serde_json::Value) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        args.to_string().hash(&mut hasher);
        hasher.finish()
    }
}

#[async_trait]
impl Detector for LoopDetector {
    fn name(&self) -> &'static str {
        "loop"
    }
    fn description(&self) -> &'static str {
        "Detects when an agent calls the same tool repeatedly without progress"
    }

    async fn detect(
        &self,
        agent_id: &str,
        observations: &[serde_json::Value],
    ) -> Vec<DetectedIssue> {
        // Count tool calls from observations
        let mut call_counts: HashMap<String, u32> = HashMap::new();
        for obs in observations {
            if let Some(tool) = obs.get("tool_name").and_then(|v| v.as_str()) {
                *call_counts.entry(tool.to_string()).or_default() += 1;
            }
        }

        // Find tools exceeding threshold
        let mut issues = Vec::new();
        for (tool_name, count) in &call_counts {
            if *count >= self.threshold {
                issues.push(DetectedIssue {
                    id: Uuid::new_v4(),
                    agent_id: agent_id.to_string(),
                    severity: if *count >= self.threshold * 2 {
                        Severity::Error
                    } else {
                        Severity::Warning
                    },
                    category: IssueCategory::LoopDetected {
                        tool_name: tool_name.clone(),
                        call_count: *count,
                        no_progress_turns: *count,
                    },
                    description: format!(
                        "Agent called '{}' {} times in {} turns — possible loop",
                        tool_name, count, self.window_turns
                    ),
                    confidence: (*count as f64 / self.threshold as f64).min(1.0),
                    suggested_actions: vec!["nudge".into(), "interject".into(), "replace".into()],
                    evidence_summary: format!(
                        "Tool '{}' called {} times, threshold={}",
                        tool_name, count, self.threshold
                    ),
                });
            }
        }

        issues
    }
}
