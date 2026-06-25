// StaleContextDetector — detects duplicate file reads and context pressure

use async_trait::async_trait;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;

pub struct StaleContextDetector {
    /// file_path → read_count
    file_reads: Mutex<HashMap<String, u32>>,
    read_threshold: u32,
    pressure_threshold: f64,
}

impl StaleContextDetector {
    pub fn new(read_threshold: u32, pressure_threshold: f64) -> Self {
        Self {
            file_reads: Mutex::new(HashMap::new()),
            read_threshold,
            pressure_threshold,
        }
    }
}

#[async_trait]
impl Detector for StaleContextDetector {
    fn name(&self) -> &'static str {
        "stale_context"
    }
    fn description(&self) -> &'static str {
        "Detects duplicate file reads and high context pressure"
    }

    async fn detect(
        &self,
        agent_id: &str,
        observations: &[serde_json::Value],
    ) -> Vec<DetectedIssue> {
        let mut issues = Vec::new();

        // Track file reads from observations
        let mut reads = self.file_reads.lock().unwrap();
        for obs in observations {
            // Check for file reads
            if let Some(file) = obs.get("file_path").and_then(|v| v.as_str()) {
                let count = reads.entry(file.to_string()).or_default();
                *count += 1;

                if *count >= self.read_threshold {
                    issues.push(DetectedIssue {
                        id: Uuid::new_v4(),
                        agent_id: agent_id.to_string(),
                        severity: if *count >= self.read_threshold * 2 {
                            Severity::Error
                        } else {
                            Severity::Warning
                        },
                        category: IssueCategory::StaleContext {
                            file_path: file.to_string(),
                            read_count: *count,
                            context_pressure: 0.0,
                        },
                        description: format!(
                            "File '{}' read {} times — context may be stale",
                            file, count
                        ),
                        confidence: (*count as f64 / self.read_threshold as f64).min(1.0),
                        suggested_actions: vec!["compact".into(), "nudge".into()],
                        evidence_summary: format!("File read {} times", count),
                    });
                }
            }

            // Check for context pressure
            if let Some(pressure) = obs.get("context_pressure").and_then(|v| v.as_f64()) {
                if pressure > self.pressure_threshold {
                    issues.push(DetectedIssue {
                        id: Uuid::new_v4(),
                        agent_id: agent_id.to_string(),
                        severity: if pressure > 0.95 {
                            Severity::Error
                        } else {
                            Severity::Warning
                        },
                        category: IssueCategory::StaleContext {
                            file_path: "n/a".into(),
                            read_count: 0,
                            context_pressure: pressure,
                        },
                        description: format!(
                            "Context pressure at {:.0}% — compaction needed",
                            pressure * 100.0
                        ),
                        confidence: pressure.min(1.0),
                        suggested_actions: vec!["compact".into()],
                        evidence_summary: format!("Context pressure: {:.0}%", pressure * 100.0),
                    });
                }
            }
        }

        issues
    }
}
