// forge-sdk/src/types/strategy.rs — Intervention strategy types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub strategy_name: String,
    pub intervention: crate::events::Intervention,
    pub priority: u32,
    pub reasoning: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyEfficacy {
    pub strategy_name: String,
    pub application_count: u64,
    pub success_rate: f64,
    pub avg_improvement: f64,
    pub make_worse_rate: f64,
}
