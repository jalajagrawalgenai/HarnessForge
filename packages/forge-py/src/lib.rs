use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass]
#[derive(Clone)]
struct HarnessRunResult {
    #[pyo3(get)]
    agent_id: String,
    #[pyo3(get)]
    success: bool,
    #[pyo3(get)]
    observation_count: u64,
    #[pyo3(get)]
    detection_count: u64,
    #[pyo3(get)]
    intervention_count: u64,
}

#[pymethods]
impl HarnessRunResult {
    fn __repr__(&self) -> String {
        format!(
            "HarnessRunResult(agent_id='{}', success={}, obs={}, det={}, int={})",
            self.agent_id,
            self.success,
            self.observation_count,
            self.detection_count,
            self.intervention_count
        )
    }
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        let _ = d.set_item("agent_id", &self.agent_id);
        let _ = d.set_item("success", self.success);
        let _ = d.set_item("observation_count", self.observation_count);
        let _ = d.set_item("detection_count", self.detection_count);
        let _ = d.set_item("intervention_count", self.intervention_count);
        Ok(d)
    }
}

#[pyclass]
struct PyHarness {
    preset: String,
}

#[pymethods]
impl PyHarness {
    #[new]
    #[pyo3(signature = (preset="solo"))]
    fn new(preset: &str) -> Self {
        Self {
            preset: preset.to_string(),
        }
    }

    fn run(&self, task: &str) -> PyResult<HarnessRunResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        rt.block_on(async {
            let mut agent =
                forge_sdk::agent::MockAgent::new("py-agent", forge_sdk::agent::AgentType::Solo)
                    .with_turns(4);
            let p = parse_preset(&self.preset);
            forge_harness::runner::run_harness_session(&mut agent, task, p, None)
                .await
                .map(|r| HarnessRunResult {
                    agent_id: r.agent_id,
                    success: r.success,
                    observation_count: r.observation_count,
                    detection_count: r.detection_count,
                    intervention_count: r.intervention_count,
                })
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    #[pyo3(signature = (task, preset="solo", turns=4))]
    fn run_with(&self, task: &str, preset: &str, turns: u32) -> PyResult<HarnessRunResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        rt.block_on(async {
            let mut agent =
                forge_sdk::agent::MockAgent::new("py-agent", forge_sdk::agent::AgentType::Solo)
                    .with_turns(turns);
            let p = parse_preset(preset);
            forge_harness::runner::run_harness_session(&mut agent, task, p, None)
                .await
                .map(|r| HarnessRunResult {
                    agent_id: r.agent_id,
                    success: r.success,
                    observation_count: r.observation_count,
                    detection_count: r.detection_count,
                    intervention_count: r.intervention_count,
                })
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn dry_run(&self, task: &str) -> PyResult<HarnessRunResult> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        rt.block_on(async {
            let mut agent =
                forge_sdk::agent::MockAgent::new("py-agent", forge_sdk::agent::AgentType::Solo)
                    .with_turns(4);
            let p = parse_preset(&self.preset);
            forge_harness::runner::dry_run(&mut agent, task, p)
                .await
                .map(|r| HarnessRunResult {
                    agent_id: r.agent_id,
                    success: r.success,
                    observation_count: r.observation_count,
                    detection_count: r.detection_count,
                    intervention_count: r.intervention_count,
                })
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
        })
    }

    fn __repr__(&self) -> String {
        format!("PyHarness(preset='{}')", self.preset)
    }
}

fn parse_preset(name: &str) -> forge_sdk::presets::Preset {
    match name.to_lowercase().as_str() {
        "solo" => forge_sdk::presets::Preset::Solo,
        "claude-code" | "claude" => forge_sdk::presets::Preset::ClaudeCode,
        "langgraph" => forge_sdk::presets::Preset::LangGraph,
        "crewai" | "crew" => forge_sdk::presets::Preset::CrewAI,
        "autogen" => forge_sdk::presets::Preset::AutoGen,
        "langchain" => forge_sdk::presets::Preset::LangChain,
        _ => forge_sdk::presets::Preset::Solo,
    }
}

#[pyfunction]
#[pyo3(signature = (preset="solo"))]
fn create_harness(preset: &str) -> PyResult<PyHarness> {
    Ok(PyHarness::new(preset))
}

#[pyfunction]
#[pyo3(signature = (task, preset="solo", turns=4))]
fn quick_run(task: &str, preset: &str, turns: u32) -> PyResult<HarnessRunResult> {
    PyHarness::new(preset).run_with(task, preset, turns)
}

#[pyfunction]
fn list_presets() -> Vec<String> {
    vec![
        "solo".into(),
        "claude-code".into(),
        "langgraph".into(),
        "crewai".into(),
        "autogen".into(),
        "langchain".into(),
    ]
}

#[pyfunction]
fn list_detectors() -> Vec<String> {
    vec![
        "loop".into(),
        "stale_context".into(),
        "cost_anomaly".into(),
        "deadlock".into(),
        "hallucination".into(),
        "prompt_injection".into(),
        "secret_leak".into(),
        "variety_collapse".into(),
        "conversation_stall".into(),
        "goal_drift".into(),
        "model_mismatch".into(),
        "accuracy_risk".into(),
        "runaway_cost".into(),
        "resource_exhaustion".into(),
        "output_degradation".into(),
        "compliance_gap".into(),
    ]
}

#[pyfunction]
fn list_strategies() -> Vec<String> {
    vec![
        "nudge".into(),
        "degrade".into(),
        "fork".into(),
        "compact".into(),
        "diversify".into(),
        "reroute".into(),
        "escalate".into(),
        "rollback".into(),
        "pause".into(),
        "interject".into(),
        "quarantine".into(),
        "replace".into(),
        "isolate".into(),
        "circuit_break".into(),
    ]
}

#[pyfunction]
fn list_observers() -> Vec<String> {
    vec![
        "token".into(),
        "latency".into(),
        "cost".into(),
        "accuracy".into(),
        "security".into(),
        "reliability".into(),
        "context_quality".into(),
        "orch".into(),
        "comm".into(),
        "compliance".into(),
        "memory".into(),
        "diversity".into(),
    ]
}

#[pyfunction]
fn get_version() -> String {
    "0.1.5".into()
}

#[pymodule]
#[pyo3(name = "forge_sdk")]
fn forge_sdk(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<HarnessRunResult>()?;
    m.add_class::<PyHarness>()?;
    m.add_function(wrap_pyfunction!(create_harness, m)?)?;
    m.add_function(wrap_pyfunction!(quick_run, m)?)?;
    m.add_function(wrap_pyfunction!(list_presets, m)?)?;
    m.add_function(wrap_pyfunction!(list_detectors, m)?)?;
    m.add_function(wrap_pyfunction!(list_strategies, m)?)?;
    m.add_function(wrap_pyfunction!(list_observers, m)?)?;
    m.add_function(wrap_pyfunction!(get_version, m)?)?;
    Ok(())
}
