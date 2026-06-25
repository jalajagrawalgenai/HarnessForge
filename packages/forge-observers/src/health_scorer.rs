// HealthScorer — aggregates all 12 dimensions into a single health score

use forge_sdk::types::health::{HealthDimensions, HealthScore, HealthTrend};

/// Aggregates dimension sub-scores into a weighted overall health score
#[derive(Clone)]
pub struct HealthScorer {
    weights: HealthWeights,
    previous_score: Option<f64>,
}

#[derive(Clone)]
pub struct HealthWeights {
    pub token: f64,
    pub latency: f64,
    pub cost: f64,
    pub accuracy: f64,
    pub orchestration: f64,
    pub communication: f64,
    pub security: f64,
    pub reliability: f64,
    pub context_quality: f64,
    pub memory: f64,
    pub compliance: f64,
    pub diversity: f64,
}

impl Default for HealthWeights {
    fn default() -> Self {
        Self {
            token: 0.12,
            latency: 0.08,
            cost: 0.10,
            accuracy: 0.12,
            orchestration: 0.08,
            communication: 0.06,
            security: 0.12,
            reliability: 0.08,
            context_quality: 0.08,
            memory: 0.04,
            compliance: 0.08,
            diversity: 0.04,
        }
    }
}

impl HealthScorer {
    pub fn new(weights: HealthWeights) -> Self {
        Self {
            weights,
            previous_score: None,
        }
    }

    /// Compute health score from dimension scores.
    /// Returns None for dimensions that are not applicable (e.g., diversity in solo mode).
    pub fn compute(&mut self, agent_id: &str, dimensions: &HealthDimensions) -> HealthScore {
        let mut total = 0.0;
        let mut weight_sum = 0.0;

        // Macro to add dimension if present
        macro_rules! add_dim {
            ($dim:ident, $weight:ident) => {
                total += dimensions.$dim * self.weights.$weight;
                weight_sum += self.weights.$weight;
            };
        }

        add_dim!(token_efficiency, token);
        add_dim!(latency, latency);
        add_dim!(cost, cost);
        add_dim!(accuracy, accuracy);
        add_dim!(orchestration, orchestration);
        add_dim!(security, security);
        add_dim!(reliability, reliability);
        add_dim!(context_quality, context_quality);
        add_dim!(compliance, compliance);

        // Optional dimensions
        if let Some(comm) = dimensions.communication {
            total += comm * self.weights.communication;
            weight_sum += self.weights.communication;
        }
        if let Some(mem) = dimensions.memory {
            total += mem * self.weights.memory;
            weight_sum += self.weights.memory;
        }
        if let Some(div) = dimensions.diversity {
            total += div * self.weights.diversity;
            weight_sum += self.weights.diversity;
        }

        let overall = if weight_sum > 0.0 {
            total / weight_sum
        } else {
            1.0
        };

        let trend = match self.previous_score {
            None => HealthTrend::Stable,
            Some(prev) if overall > prev + 0.05 => HealthTrend::Improving,
            Some(prev) if overall < prev - 0.05 => HealthTrend::Degrading,
            _ => HealthTrend::Stable,
        };

        self.previous_score = Some(overall);

        HealthScore {
            agent_id: agent_id.to_string(),
            overall,
            dimensions: dimensions.clone(),
            trend,
        }
    }
}
