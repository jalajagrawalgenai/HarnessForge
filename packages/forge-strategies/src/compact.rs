// CompactStrategy — trigger context compaction

use async_trait::async_trait;
use forge_sdk::events::{CompressionLayer, Intervention};
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::DetectedIssue;
use forge_sdk::types::strategy::StrategyResult;
use forge_sdk::types::detection::IssueCategory;

pub struct CompactStrategy {
    target_ratio: f64,
}

impl CompactStrategy {
    pub fn new(target_ratio: f64) -> Self {
        Self { target_ratio }
    }

    fn select_layer(pressure: f64) -> CompressionLayer {
        if pressure > 0.95 {
            CompressionLayer::Autocompact
        } else if pressure > 0.85 {
            CompressionLayer::Microcompact
        } else if pressure > 0.70 {
            CompressionLayer::Snip
        } else {
            CompressionLayer::Budget
        }
    }
}

#[async_trait]
impl Strategy for CompactStrategy {
    fn name(&self) -> &'static str { "compact" }
    fn priority(&self) -> u32 { 20 }

    async fn evaluate(&self, detection: &DetectedIssue) -> Option<StrategyResult> {
        let pressure = match &detection.category {
            IssueCategory::StaleContext { context_pressure, .. } => *context_pressure,
            _ => return None, // Only handle stale context
        };

        let layer = Self::select_layer(pressure);
        let layer_clone = layer.clone();
        let intervention = Intervention::Compact {
            target_ratio: self.target_ratio,
            layer,
        };

        Some(StrategyResult {
            strategy_name: "compact".into(),
            intervention,
            priority: self.priority(),
            reasoning: format!(
                "Context pressure at {:.0}%, applying {:?} compaction to reach {:.0}%",
                pressure * 100.0, layer_clone, self.target_ratio * 100.0
            ),
            confidence: detection.confidence,
        })
    }
}
