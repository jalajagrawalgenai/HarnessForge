use async_trait::async_trait;
use uuid::Uuid;
use forge_sdk::traits::detector::Detector;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};

pub struct ConversationStallDetector { timeout_secs: u64 }
impl ConversationStallDetector { pub fn new(t: u64) -> Self { Self { timeout_secs: t } } }

#[async_trait]
impl Detector for ConversationStallDetector {
    fn name(&self) -> &'static str { "conversation_stall" }
    fn description(&self) -> &'static str { "Detects when multi-agent conversation has stopped" }
    async fn detect(&self, agent_id: &str, obs: &[serde_json::Value]) -> Vec<DetectedIssue> {
        let mut last_msg: Option<u64> = None;
        for o in obs {
            if let Some(ts) = o.get("msg_timestamp_ms").and_then(|v| v.as_u64()) {
                last_msg = Some(ts);
            }
        }
        if let Some(ts) = last_msg {
            let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() * 1000;
            let elapsed = (now.saturating_sub(ts)) / 1000;
            if elapsed >= self.timeout_secs {
                vec![DetectedIssue { id: Uuid::new_v4(), agent_id: agent_id.into(),
                    severity: if elapsed > self.timeout_secs * 2 { Severity::Error } else { Severity::Warning },
                    category: IssueCategory::ConversationStall { duration_secs: elapsed },
                    description: format!("Conversation stalled for {}s", elapsed),
                    confidence: (elapsed as f64 / self.timeout_secs as f64).min(1.0),
                    suggested_actions: vec!["nudge".into(), "change_speaker".into()],
                    evidence_summary: format!("No messages for {}s", elapsed) }]
            } else { vec![] }
        } else { vec![] }
    }
}
