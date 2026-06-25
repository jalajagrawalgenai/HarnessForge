use pyo3::prelude::*;

#[pyclass]
pub struct PyHarness { config: String }

#[pymethods]
impl PyHarness {
    #[new]
    fn new(config: Option<String>) -> Self { Self { config: config.unwrap_or_default() } }
    fn run(&self, task: &str) -> PyResult<String> { Ok(format!("Harness ran: {}", task)) }
    fn health(&self) -> PyResult<f64> { Ok(0.95) }
    fn __repr__(&self) -> String { format!("PyHarness(config={})", self.config) }
}

#[pyfunction]
fn create_harness(preset: Option<String>) -> PyResult<PyHarness> {
    Ok(PyHarness { config: preset.unwrap_or_else(|| "solo".into()) })
}

#[pyfunction]
fn list_detectors() -> PyResult<Vec<String>> {
    Ok(vec!["loop".into(),"secret_leak".into(),"stale_context".into(),"hallucination".into(),"cost_anomaly".into(),"deadlock".into(),"prompt_injection".into(),"variety_collapse".into()])
}

#[pyfunction]
fn list_strategies() -> PyResult<Vec<String>> {
    Ok(vec!["nudge".into(),"compact".into(),"pause".into(),"escalate".into(),"circuit_break".into(),"isolate".into(),"replace".into(),"quarantine".into()])
}

#[pyfunction]
fn get_version() -> PyResult<String> { Ok("0.1.0".into()) }

#[pymodule]
fn forge_sdk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyHarness>()?;
    m.add_function(wrap_pyfunction!(create_harness, m)?)?;
    m.add_function(wrap_pyfunction!(list_detectors, m)?)?;
    m.add_function(wrap_pyfunction!(list_strategies, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    Ok(())
}
