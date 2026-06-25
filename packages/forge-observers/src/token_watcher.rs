// TokenWatcher — tracks token efficiency

use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct TokenWatcher {
    cache_hits: Mutex<u64>,
    cache_misses: Mutex<u64>,
    dedup_hits: Mutex<u64>,
    total_tool_calls: Mutex<u64>,
    waste_tokens: Mutex<u64>,
    tokens_per_turn: Mutex<Vec<u64>>,
    _tokens_per_tool: Mutex<std::collections::HashMap<String, Vec<u64>>>,
}

#[allow(clippy::new_without_default)]
impl TokenWatcher {
    pub fn new() -> Self {
        Self {
            cache_hits: Mutex::new(0),
            cache_misses: Mutex::new(0),
            dedup_hits: Mutex::new(0),
            total_tool_calls: Mutex::new(0),
            waste_tokens: Mutex::new(0),
            tokens_per_turn: Mutex::new(Vec::new()),
            _tokens_per_tool: Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub fn cache_hit_rate(&self) -> f64 {
        let hits = *self.cache_hits.lock().unwrap() as f64;
        let misses = *self.cache_misses.lock().unwrap() as f64;
        let total = hits + misses;
        if total == 0.0 { 0.0 } else { hits / total }
    }

    pub fn dedup_rate(&self) -> f64 {
        let dedup = *self.dedup_hits.lock().unwrap() as f64;
        let total = *self.total_tool_calls.lock().unwrap() as f64;
        if total == 0.0 { 0.0 } else { dedup / total }
    }

    pub fn waste_tokens(&self) -> u64 {
        *self.waste_tokens.lock().unwrap()
    }

    pub fn avg_tokens_per_turn(&self) -> f64 {
        let turns = self.tokens_per_turn.lock().unwrap();
        if turns.is_empty() { 0.0 } else {
            turns.iter().sum::<u64>() as f64 / turns.len() as f64
        }
    }
}

#[async_trait]
impl Observer for TokenWatcher {
    fn name(&self) -> &'static str { "token" }
    fn dimension(&self) -> &'static str { "token_efficiency" }

    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::TokenUsage { input, output, cache_read, cache_write, .. } => {
                if *cache_read > 0 {
                    *self.cache_hits.lock().unwrap() += 1;
                } else {
                    *self.cache_misses.lock().unwrap() += 1;
                }
                let total = input + output + cache_read + cache_write;
                self.tokens_per_turn.lock().unwrap().push(total);

                Some(serde_json::json!({
                    "dimension": "token",
                    "total_tokens": total,
                    "cache_hit_rate": self.cache_hit_rate(),
                    "dedup_rate": self.dedup_rate(),
                    "waste_tokens": self.waste_tokens(),
                }))
            }
            AgentEvent::ToolCallCached { .. } => {
                *self.dedup_hits.lock().unwrap() += 1;
                None
            }
            AgentEvent::ToolCallStart { .. } => {
                *self.total_tool_calls.lock().unwrap() += 1;
                None
            }
            _ => None,
        }
    }
}
