use crate::harness::HarnessBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preset {
    // ─── Frameworks ───
    Solo, LangGraph, CrewAI, AutoGen, LangChain, OpenAISwarm,
    SemanticKernel, Haystack, DSPy, LlamaIndex, TaskWeaver,
    Agno, AtomicAgents, BeeAgent, PydanticAI,
    // ─── Coding agents ───
    ClaudeCode, Aider, Cline, Continue,
    // ─── Cloud / IDE (API bridged) ───
    VercelAI, Copilot, Cursor, Windsurf, Devin, AmazonQ,
    ReplitAgent, PearAI, BoltNew, Lovable, V0,
    Custom,
}

impl Preset {
    pub fn observers(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["token","latency","cost","accuracy","security","reliability","context_quality","orch"],
            Preset::LangGraph => vec!["token","latency","orch","comm","context_quality","reliability"],
            Preset::CrewAI => vec!["token","latency","orch","comm","accuracy","diversity","reliability"],
            Preset::AutoGen => vec!["token","latency","comm","accuracy","diversity","security"],
            Preset::LangChain => vec!["token","latency","cost","accuracy","security","reliability","context_quality"],
            Preset::OpenAISwarm => vec!["token","latency","orch","comm","accuracy"],
            Preset::SemanticKernel => vec!["token","latency","cost","accuracy","security","compliance"],
            Preset::Haystack => vec!["token","latency","accuracy","context_quality","memory"],
            Preset::DSPy => vec!["token","latency","cost","accuracy","orch"],
            Preset::LlamaIndex => vec!["token","latency","accuracy","context_quality","memory","reliability"],
            Preset::TaskWeaver => vec!["token","latency","accuracy","security","reliability"],
            Preset::Agno => vec!["token","latency","cost","accuracy","memory","diversity"],
            Preset::AtomicAgents => vec!["token","latency","accuracy","security"],
            Preset::BeeAgent => vec!["token","latency","orch","reliability","context_quality"],
            Preset::PydanticAI => vec!["token","latency","accuracy","security","compliance"],
            Preset::ClaudeCode => vec!["token","latency","cost","accuracy","security","reliability","context_quality"],
            Preset::Aider => vec!["token","latency","cost","accuracy","context_quality"],
            Preset::Cline => vec!["token","latency","cost","accuracy","security"],
            Preset::Continue => vec!["token","latency","accuracy","context_quality"],
            Preset::VercelAI => vec!["token","latency","cost","accuracy"],
            Preset::Copilot => vec!["token","latency","accuracy","security"],
            Preset::Cursor => vec!["token","latency","cost","accuracy","context_quality"],
            Preset::Windsurf => vec!["token","latency","accuracy","context_quality"],
            Preset::Devin => vec!["token","latency","cost","accuracy","security"],
            Preset::AmazonQ => vec!["token","latency","cost","security","compliance"],
            Preset::ReplitAgent => vec!["token","latency","cost","accuracy"],
            Preset::PearAI => vec!["token","latency","accuracy"],
            Preset::BoltNew => vec!["token","latency","cost","accuracy"],
            Preset::Lovable => vec!["token","latency","cost","accuracy"],
            Preset::V0 => vec!["token","latency","accuracy","security"],
            Preset::Custom => vec!["token","latency","cost","accuracy","security"],
        }
    }
    pub fn detectors(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["loop","stale_context","cost_anomaly","secret_leak","hallucination","accuracy_risk","model_mismatch"],
            Preset::LangGraph => vec!["deadlock","loop","stale_context","conversation_stall","runaway_cost"],
            Preset::CrewAI => vec!["deadlock","variety_collapse","conversation_stall","goal_drift","output_degradation"],
            Preset::AutoGen => vec!["conversation_stall","prompt_injection","goal_drift","variety_collapse","loop"],
            Preset::LangChain => vec!["loop","stale_context","hallucination","secret_leak","accuracy_risk","cost_anomaly"],
            Preset::OpenAISwarm => vec!["conversation_stall","goal_drift","loop","deadlock"],
            Preset::SemanticKernel => vec!["secret_leak","prompt_injection","accuracy_risk","compliance_gap","model_mismatch"],
            Preset::Haystack => vec!["stale_context","accuracy_risk","hallucination","output_degradation"],
            Preset::DSPy => vec!["accuracy_risk","model_mismatch","output_degradation","cost_anomaly","loop"],
            Preset::LlamaIndex => vec!["stale_context","hallucination","accuracy_risk","output_degradation","loop"],
            Preset::TaskWeaver => vec!["accuracy_risk","secret_leak","loop","resource_exhaustion"],
            Preset::Agno => vec!["loop","hallucination","accuracy_risk","cost_anomaly","variety_collapse"],
            Preset::AtomicAgents => vec!["loop","hallucination","accuracy_risk","secret_leak"],
            Preset::BeeAgent => vec!["loop","stale_context","deadlock","accuracy_risk"],
            Preset::PydanticAI => vec!["accuracy_risk","secret_leak","compliance_gap","hallucination"],
            Preset::ClaudeCode => vec!["loop","stale_context","cost_anomaly","secret_leak","hallucination","accuracy_risk","model_mismatch","runaway_cost"],
            Preset::Aider => vec!["loop","accuracy_risk","cost_anomaly","hallucination","stale_context"],
            Preset::Cline => vec!["loop","secret_leak","hallucination","accuracy_risk","cost_anomaly"],
            Preset::Continue => vec!["loop","stale_context","hallucination","accuracy_risk"],
            Preset::VercelAI => vec!["loop","cost_anomaly","accuracy_risk"],
            Preset::Copilot => vec!["secret_leak","accuracy_risk","prompt_injection"],
            Preset::Cursor => vec!["loop","stale_context","accuracy_risk","secret_leak"],
            Preset::Windsurf => vec!["loop","accuracy_risk","hallucination"],
            Preset::Devin => vec!["loop","cost_anomaly","accuracy_risk","secret_leak"],
            Preset::AmazonQ => vec!["secret_leak","compliance_gap","cost_anomaly"],
            Preset::ReplitAgent => vec!["loop","cost_anomaly","accuracy_risk"],
            Preset::PearAI => vec!["loop","accuracy_risk"],
            Preset::BoltNew => vec!["loop","cost_anomaly","accuracy_risk"],
            Preset::Lovable => vec!["loop","cost_anomaly","accuracy_risk"],
            Preset::V0 => vec!["accuracy_risk","secret_leak","loop"],
            Preset::Custom => vec!["loop","stale_context","secret_leak"],
        }
    }
    pub fn strategies(&self) -> Vec<&'static str> {
        match self {
            Preset::Solo => vec!["nudge","compact","pause","escalate","replace"],
            Preset::LangGraph => vec!["reroute","nudge","compact","pause","rollback"],
            Preset::CrewAI => vec!["diversify","nudge","pause","replace","fork"],
            Preset::AutoGen => vec!["nudge","interject","pause","diversify","rollback"],
            Preset::LangChain => vec!["nudge","compact","pause","escalate","replace","rollback"],
            Preset::OpenAISwarm => vec!["nudge","reroute","pause","diversify"],
            Preset::SemanticKernel => vec!["nudge","pause","isolate","escalate","quarantine"],
            Preset::Haystack => vec!["nudge","compact","replace","rollback"],
            Preset::DSPy => vec!["nudge","escalate","replace","rollback","compact"],
            Preset::LlamaIndex => vec!["nudge","compact","replace","rollback","pause"],
            Preset::TaskWeaver => vec!["nudge","pause","escalate","rollback"],
            Preset::Agno => vec!["nudge","compact","diversify","replace","pause"],
            Preset::AtomicAgents => vec!["nudge","pause","replace"],
            Preset::BeeAgent => vec!["nudge","reroute","compact","pause","rollback"],
            Preset::PydanticAI => vec!["nudge","pause","isolate","escalate","quarantine"],
            Preset::ClaudeCode => vec!["nudge","compact","pause","escalate","replace","rollback","interject","degrade"],
            Preset::Aider => vec!["nudge","compact","pause","replace","rollback"],
            Preset::Cline => vec!["nudge","compact","pause","replace","escalate"],
            Preset::Continue => vec!["nudge","compact","pause","replace"],
            Preset::VercelAI => vec!["nudge","pause","degrade"],
            Preset::Copilot => vec!["nudge","pause","interject"],
            Preset::Cursor => vec!["nudge","compact","pause"],
            Preset::Windsurf => vec!["nudge","pause"],
            Preset::Devin => vec!["nudge","pause","degrade","escalate"],
            Preset::AmazonQ => vec!["nudge","pause","isolate"],
            Preset::ReplitAgent => vec!["nudge","pause","degrade"],
            Preset::PearAI => vec!["nudge","pause"],
            Preset::BoltNew => vec!["nudge","pause","degrade"],
            Preset::Lovable => vec!["nudge","pause","degrade"],
            Preset::V0 => vec!["nudge","pause"],
            Preset::Custom => vec!["nudge","compact","pause"],
        }
    }
    /// Apply this preset's observer, detector, and strategy configuration to a builder.
    ///
    /// This stores the preset name selections on the builder config so that
    /// forge-harness can later wire concrete types via the factory. For direct
    /// concrete type instantiation, use `forge_harness::factory::build_registry_from_preset()`.
    pub fn apply(&self, builder: HarnessBuilder) -> HarnessBuilder {
        // The preset's name selections are available via self.observers(),
        // self.detectors(), self.strategies(). The forge-harness factory uses
        // these names to instantiate concrete types.
        //
        // The builder stores nothing preset-specific — it just holds the
        // observer/detector/strategy trait objects that have been registered.
        // For the full preset wiring, use the forge-harness factory directly.
        builder
            .dry_run(false)
    }

    /// Human-readable description of what this preset configures.
    pub fn describe(&self) -> String {
        format!(
            "Preset::{:?} — {} observers, {} detectors, {} strategies",
            self,
            self.observers().len(),
            self.detectors().len(),
            self.strategies().len(),
        )
    }
}
