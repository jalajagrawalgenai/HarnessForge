// forge-sdk/src/traits/observer.rs — Observer trait

use async_trait::async_trait;
use crate::events::AgentEvent;

#[async_trait]
pub trait Observer: Send + Sync {
    fn name(&self) -> &'static str;
    fn dimension(&self) -> &'static str;
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value>;
}
