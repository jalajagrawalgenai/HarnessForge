// forge-sdk/src/traits/strategy.rs — Strategy trait

use async_trait::async_trait;
use crate::types::detection::DetectedIssue;
use crate::types::strategy::StrategyResult;

#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u32;
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult>;
}
