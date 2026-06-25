// forge-sdk/src/traits/strategy.rs — Strategy trait

use crate::types::detection::DetectedIssue;
use crate::types::strategy::StrategyResult;
use async_trait::async_trait;

#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn priority(&self) -> u32;
    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult>;
}
