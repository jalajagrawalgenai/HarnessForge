// forge-sdk/src/types/token.rs — Token and cost types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCount {
    pub input: u64,
    pub output: u64,
    pub cache_write: u64,
    pub cache_read: u64,
}

impl TokenCount {
    pub fn total(&self) -> u64 {
        self.input + self.output + self.cache_write + self.cache_read
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let total_cache = self.cache_read + self.cache_write;
        if total_cache == 0 {
            return 0.0;
        }
        self.cache_read as f64 / total_cache as f64
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_total: u64,
    pub max_per_turn: u64,
    pub warn_at: f64,
    pub critical_at: f64,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            max_total: 200_000,
            max_per_turn: 64_000,
            warn_at: 0.7,
            critical_at: 0.9,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetrics {
    pub cache_hit_rate: f64,
    pub dedup_rate: f64,
    pub compression_ratio: f64,
    pub waste_tokens: u64,
    pub tokens_per_turn: f64,
    pub tokens_per_tool: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostMetrics {
    pub cost_per_turn: f64,
    pub cost_per_operation: f64,
    pub budget_burn_rate: f64,
    pub cost_efficiency: f64,
    pub projected_total_cost: f64,
    pub model_cost_vs_cheaper: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMetrics {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub ttft_ms: f64,
    pub tool_avg_ms: f64,
    pub turn_avg_ms: f64,
    pub trend: f64, // positive = slowing down
}
