use forge_bridge::cost::CostCalculator;
use forge_bridge::models::ModelCatalog;

#[test]
fn test_model_catalog_default() {
    let catalog = ModelCatalog::default_catalog();
    assert!(catalog.models.len() >= 5);
    assert!(catalog.find("claude-sonnet-4-6").is_some());
    assert!(catalog.find("nonexistent").is_none());
}

#[test]
fn test_cheapest_for() {
    let catalog = ModelCatalog::default_catalog();
    let cheapest = catalog.cheapest_for("fast");
    assert!(cheapest.is_some());
}

#[test]
fn test_cost_calculator() {
    let catalog = ModelCatalog::default_catalog();
    let calc = CostCalculator::new(catalog);
    let cost = calc.calculate("claude-sonnet-4-6", 10000, 2000, 500, 100);
    assert!(cost > 0.0);
}

#[test]
fn test_cost_estimate() {
    let catalog = ModelCatalog::default_catalog();
    let calc = CostCalculator::new(catalog);
    let est = calc.estimate("claude-haiku-4-5", 5000, 500);
    assert!(est > 0.0);
    assert!(est < 1.0);
}
