use async_trait::async_trait;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use uuid::Uuid;

pub struct ResourceExhaustionDetector {
    disk_pct: f64,
    mem_pct: f64,
}
impl ResourceExhaustionDetector {
    pub fn new(disk: f64, mem: f64) -> Self {
        Self {
            disk_pct: disk,
            mem_pct: mem,
        }
    }
}

#[async_trait]
impl Detector for ResourceExhaustionDetector {
    fn name(&self) -> &'static str {
        "resource_exhaustion"
    }
    fn description(&self) -> &'static str {
        "Detects disk/memory/network exhaustion"
    }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let mut issues = Vec::new();
        for o in obs {
            if let (Some(res), Some(pct)) = (
                o.get("resource").and_then(|v| v.as_str()),
                o.get("usage_pct").and_then(|v| v.as_f64()),
            ) {
                let threshold = if res == "disk" {
                    self.disk_pct
                } else {
                    self.mem_pct
                };
                if pct > threshold {
                    issues.push(DetectedIssue {
                        id: Uuid::new_v4(),
                        agent_id: agent_id.into(),
                        severity: if pct > 0.95 {
                            Severity::Error
                        } else {
                            Severity::Warning
                        },
                        category: IssueCategory::ResourceExhaustion {
                            resource: res.into(),
                            usage_pct: pct,
                        },
                        description: format!("{} usage at {:.0}%", res, pct * 100.0),
                        confidence: (pct / threshold).min(1.0),
                        suggested_actions: vec!["compact".into(), "pause".into()],
                        evidence_summary: format!("{}: {:.0}%", res, pct * 100.0),
                    });
                }
            }
        }
        issues
    }
}
