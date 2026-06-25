use crate::models::ModelCatalog;

pub struct CostCalculator { catalog: ModelCatalog }

impl CostCalculator {
    pub fn new(catalog: ModelCatalog) -> Self { Self { catalog } }

    pub fn calculate(&self, model: &str, input_tokens: u64, output_tokens: u64, cache_write: u64, cache_read: u64) -> f64 {
        let entry = self.catalog.find(model);
        match entry {
            Some(e) => {
                (input_tokens as f64 / 1000.0) * e.input_cost_per_1k
                    + (output_tokens as f64 / 1000.0) * e.output_cost_per_1k
                    + (cache_write as f64 / 1000.0) * e.cache_write_cost_per_1k
                    + (cache_read as f64 / 1000.0) * e.cache_read_cost_per_1k
            }
            None => 0.0,
        }
    }

    pub fn estimate(&self, model: &str, estimated_input: u64, estimated_output: u64) -> f64 {
        self.calculate(model, estimated_input, estimated_output, 0, 0)
    }
}
