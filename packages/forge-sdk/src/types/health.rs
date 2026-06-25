// forge-sdk/src/types/health.rs — Health score types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthScore {
    pub agent_id: String,
    pub overall: f64,
    pub dimensions: HealthDimensions,
    pub trend: HealthTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthDimensions {
    pub token_efficiency: f64,
    pub latency: f64,
    pub cost: f64,
    pub accuracy: f64,
    pub orchestration: f64,
    pub communication: Option<f64>,
    pub security: f64,
    pub reliability: f64,
    pub context_quality: f64,
    pub memory: Option<f64>,
    pub compliance: f64,
    pub diversity: Option<f64>,
}

impl Default for HealthDimensions {
    fn default() -> Self {
        Self {
            token_efficiency: 1.0,
            latency: 1.0,
            cost: 1.0,
            accuracy: 1.0,
            orchestration: 1.0,
            communication: None,
            security: 1.0,
            reliability: 1.0,
            context_quality: 1.0,
            memory: None,
            compliance: 1.0,
            diversity: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthTrend {
    Improving,
    Stable,
    Degrading,
}

impl HealthScore {
    pub fn color(&self) -> &'static str {
        if self.overall > 0.8 {
            "green"
        } else if self.overall > 0.5 {
            "yellow"
        } else if self.overall > 0.3 {
            "orange"
        } else {
            "red"
        }
    }

    pub fn emoji(&self) -> &'static str {
        if self.overall > 0.8 {
            "🟢"
        } else if self.overall > 0.5 {
            "🟡"
        } else if self.overall > 0.3 {
            "🟠"
        } else {
            "🔴"
        }
    }
}
