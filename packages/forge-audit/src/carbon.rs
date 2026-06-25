pub struct CarbonTracker { total_tokens: u64, total_seconds: f64 }

impl CarbonTracker {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { total_tokens: 0, total_seconds: 0.0 } }
    pub fn record(&mut self, tokens: u64, duration_secs: f64) { self.total_tokens += tokens; self.total_seconds += duration_secs; }
    pub fn estimate_kg_co2(&self) -> f64 {
        let token_energy = self.total_tokens as f64 * 0.0000003; // ~0.3 Wh per 1000 tokens
        let compute_energy = self.total_seconds * 0.00015; // ~0.15 kWh per compute-hour
        let total_kwh = token_energy + compute_energy / 3600.0;
        total_kwh * 0.4 // ~0.4 kg CO2 per kWh (global average)
    }
    pub fn report(&self) -> String {
        format!("Carbon: {:.4} kg CO2e ({} tokens, {:.0}s compute)", self.estimate_kg_co2(), self.total_tokens, self.total_seconds)
    }
}
