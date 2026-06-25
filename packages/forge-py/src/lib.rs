use pyo3::prelude::*;

#[pyfunction]
fn run_agent(task: &str, config: &str) -> PyResult<String> {
    Ok(format!("Forge agent ran task '{}' with config '{}'", task, config))
}

#[pyfunction]
fn get_version() -> PyResult<String> { Ok("0.1.0".into()) }

#[pymodule]
fn forge_sdk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(run_agent, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    Ok(())
}
