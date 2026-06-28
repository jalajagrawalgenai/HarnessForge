use async_trait::async_trait;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;
use std::sync::Mutex;

pub struct CostWatcher {
    total_cost: Mutex<f64>,
    turn_costs: Mutex<Vec<f64>>,
}

#[allow(clippy::new_without_default)]
impl CostWatcher {
    pub fn new() -> Self {
        Self {
            total_cost: Mutex::new(0.0),
            turn_costs: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait]
impl Observer for CostWatcher {
    fn name(&self) -> &'static str {
        "cost"
    }
    fn dimension(&self) -> &'static str {
        "cost"
    }
    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        if let AgentEvent::TokenUsage {
            input,
            output,
            model,
            ..
        } = event
        {
            let est_cost = (*input as f64 * 3.0 + *output as f64 * 15.0) / 1_000_000.0;
            let mut total = self.total_cost.lock().unwrap();
            *total += est_cost;
            self.turn_costs.lock().unwrap().push(est_cost);
            Some(
                serde_json::json!({
                    "dimension":"cost",
                    "turn_cost":est_cost,
                    "cost_per_turn":est_cost,  // alias for CostAnomaly + RunawayCost detectors
                    "total_cost":*total,
                    "model":model,
                }),
            )
        } else {
            None
        }
    }
}
