use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCatalog { pub models: Vec<ModelEntry> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub provider: String,
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub cache_write_cost_per_1k: f64,
    pub cache_read_cost_per_1k: f64,
    pub context_window: u64,
    pub max_output: u64,
    pub capabilities: Vec<String>,
}

impl ModelCatalog {
    pub fn default_catalog() -> Self {
        Self { models: vec![
            ModelEntry { id: "claude-opus-4-8".into(), provider: "anthropic".into(), input_cost_per_1k: 0.015, output_cost_per_1k: 0.075, cache_write_cost_per_1k: 0.01875, cache_read_cost_per_1k: 0.0015, context_window: 200000, max_output: 32000, capabilities: vec!["reasoning".into(), "code".into(), "tools".into()] },
            ModelEntry { id: "claude-sonnet-4-6".into(), provider: "anthropic".into(), input_cost_per_1k: 0.003, output_cost_per_1k: 0.015, cache_write_cost_per_1k: 0.00375, cache_read_cost_per_1k: 0.0003, context_window: 200000, max_output: 64000, capabilities: vec!["reasoning".into(), "code".into(), "tools".into()] },
            ModelEntry { id: "claude-haiku-4-5".into(), provider: "anthropic".into(), input_cost_per_1k: 0.0008, output_cost_per_1k: 0.004, cache_write_cost_per_1k: 0.001, cache_read_cost_per_1k: 0.00008, context_window: 200000, max_output: 32000, capabilities: vec!["fast".into(), "tools".into()] },
            ModelEntry { id: "gpt-4o".into(), provider: "openai".into(), input_cost_per_1k: 0.0025, output_cost_per_1k: 0.01, cache_write_cost_per_1k: 0.0, cache_read_cost_per_1k: 0.00125, context_window: 128000, max_output: 16384, capabilities: vec!["reasoning".into(), "code".into(), "vision".into()] },
            ModelEntry { id: "gemini-2.0-flash".into(), provider: "google".into(), input_cost_per_1k: 0.0001, output_cost_per_1k: 0.0004, cache_write_cost_per_1k: 0.0, cache_read_cost_per_1k: 0.0, context_window: 1000000, max_output: 32000, capabilities: vec!["fast".into(), "multimodal".into()] },
        ]}
    }

    pub fn find(&self, model_id: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|m| m.id == model_id)
    }

    pub fn cheapest_for(&self, capability: &str) -> Option<&ModelEntry> {
        self.models.iter()
            .filter(|m| m.capabilities.iter().any(|c| c == capability))
            .min_by(|a, b| a.input_cost_per_1k.partial_cmp(&b.input_cost_per_1k).unwrap())
    }
}
