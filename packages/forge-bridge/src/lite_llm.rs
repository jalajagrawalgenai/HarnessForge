use crate::cost::CostCalculator;
use crate::models::ModelCatalog;

#[allow(dead_code)]
pub struct LiteLlmClient {
    base_url: String,
    api_key: Option<String>,
    catalog: ModelCatalog,
    calculator: CostCalculator,
}

impl LiteLlmClient {
    pub fn new(base_url: &str, api_key: Option<&str>) -> Self {
        let catalog = ModelCatalog::default_catalog();
        let calculator = CostCalculator::new(catalog.clone());
        Self {
            base_url: base_url.into(),
            api_key: api_key.map(String::from),
            catalog,
            calculator,
        }
    }

    pub fn catalog(&self) -> &ModelCatalog {
        &self.catalog
    }
    pub fn calculator(&self) -> &CostCalculator {
        &self.calculator
    }

    pub fn estimate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        self.calculator
            .calculate(model, input_tokens, output_tokens, 0, 0)
    }
}
