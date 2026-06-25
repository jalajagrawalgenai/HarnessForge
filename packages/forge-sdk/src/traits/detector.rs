// forge-sdk/src/traits/detector.rs — Detector trait

use async_trait::async_trait;
use crate::types::detection::DetectedIssue;

#[async_trait]
pub trait Detector: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    async fn detect(
        &self,
        agent_id: &str,
        observations: &[serde_json::Value],
    ) -> Vec<DetectedIssue>;
}
