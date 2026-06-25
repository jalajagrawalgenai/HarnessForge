use crate::harness::HarnessBuilder;

pub enum Preset { Solo, LangGraph, CrewAI, AutoGen, Custom }

impl Preset {
    pub fn observers(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["token","latency","cost","accuracy","security","reliability","context_quality","orch"],
            Preset::LangGraph => vec!["token","latency","orch","comm","context_quality"],
            Preset::CrewAI => vec!["token","latency","orch","comm","accuracy","diversity"],
            Preset::AutoGen => vec!["token","latency","comm","accuracy","diversity","security"],
            Preset::Custom => vec!["token","latency","cost","accuracy","security"],
        }
    }
    pub fn detectors(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["loop","stale_context","cost_anomaly","secret_leak","hallucination","accuracy_risk","model_mismatch"],
            Preset::LangGraph => vec!["deadlock","loop","stale_context","conversation_stall"],
            Preset::CrewAI => vec!["deadlock","variety_collapse","conversation_stall","goal_drift"],
            Preset::AutoGen => vec!["conversation_stall","prompt_injection","goal_drift","variety_collapse"],
            Preset::Custom => vec!["loop","stale_context","secret_leak"],
        }
    }
    pub fn strategies(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["nudge","compact","pause","escalate","replace"],
            Preset::LangGraph => vec!["reroute","nudge","compact","pause"],
            Preset::CrewAI => vec!["diversify","nudge","pause","replace"],
            Preset::AutoGen => vec!["nudge","interject","pause","diversify"],
            Preset::Custom => vec!["nudge","compact","pause"],
        }
    }
    pub fn apply(&self, builder: HarnessBuilder) -> HarnessBuilder { builder }
}
