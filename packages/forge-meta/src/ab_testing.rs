use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct ABTestEngine { tests: Vec<ABTest> }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    pub name: String,
    pub control_pass_rate: f64,
    pub treatment_pass_rate: f64,
    pub control_tokens: u64,
    pub treatment_tokens: u64,
    pub control_cost: f64,
    pub treatment_cost: f64,
    pub sample_size: usize,
    pub p_value: f64,
    pub winner: Option<ABWinner>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ABWinner { Control, Treatment, Tie }

impl ABTestEngine {
    pub fn new() -> Self { Self::default() }

    pub fn run(
        &mut self,
        name: &str,
        control_metrics: &TestMetrics,
        treatment_metrics: &TestMetrics,
        sample_size: usize,
    ) -> &ABTest {
        let pass_delta = treatment_metrics.pass_rate - control_metrics.pass_rate;
        let p_value = if sample_size > 10 {
            let se = ((control_metrics.pass_rate * (1.0 - control_metrics.pass_rate)
                + treatment_metrics.pass_rate * (1.0 - treatment_metrics.pass_rate))
                / sample_size as f64).sqrt();
            if se > 0.0 {
                (1.0 - normal_cdf(pass_delta.abs() / se)) * 2.0
            } else { 1.0 }
        } else { 0.5 };

        let winner = if p_value < 0.05 && pass_delta > 0.03 {
            Some(ABWinner::Treatment)
        } else if p_value < 0.05 && pass_delta < -0.03 {
            Some(ABWinner::Control)
        } else {
            Some(ABWinner::Tie)
        };

        let test = ABTest {
            name: name.into(),
            control_pass_rate: control_metrics.pass_rate,
            treatment_pass_rate: treatment_metrics.pass_rate,
            control_tokens: control_metrics.avg_tokens,
            treatment_tokens: treatment_metrics.avg_tokens,
            control_cost: control_metrics.avg_cost,
            treatment_cost: treatment_metrics.avg_cost,
            sample_size,
            p_value,
            winner,
        };

        self.tests.push(test);
        self.tests.last().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct TestMetrics {
    pub pass_rate: f64,
    pub avg_tokens: u64,
    pub avg_cost: f64,
}

fn normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + libm::erf(x / 1.4142135623730951))
}
