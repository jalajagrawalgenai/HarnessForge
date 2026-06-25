// forge-harness/src/factory.rs — Build PluginRegistry from Preset
//
// Wiring layer: maps Preset string names → concrete observer/detector/strategy
// instances from forge-observers, forge-detectors, forge-strategies.

use std::sync::Arc;
use forge_sdk::presets::Preset;
use crate::plugin_registry::PluginRegistry;

/// Build a fully-populated PluginRegistry from a Preset.
pub fn build_registry_from_preset(preset: &Preset) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    register_observers(&mut registry, &preset.observers());
    register_detectors(&mut registry, &preset.detectors());
    register_strategies(&mut registry, &preset.strategies());
    registry
}

/// Build a PluginRegistry from explicit string lists (Custom preset, CLI flags).
pub fn build_registry_from_names(
    observer_names: &[&str],
    detector_names: &[&str],
    strategy_names: &[&str],
) -> PluginRegistry {
    let mut registry = PluginRegistry::new();
    register_observers(&mut registry, observer_names);
    register_detectors(&mut registry, detector_names);
    register_strategies(&mut registry, strategy_names);
    registry
}

// ─── Observers (12 watchers) ──────────────────────────────────────────
// All implement forge_sdk::traits::observer::Observer
// All have `pub fn new() -> Self` (zero-arg constructors)

fn register_observers(registry: &mut PluginRegistry, names: &[&str]) {
    for name in names {
        match *name {
            "token" => registry.register_observer(Arc::new(
                forge_observers::token_watcher::TokenWatcher::new(),
            )),
            "latency" => registry.register_observer(Arc::new(
                forge_observers::latency_watcher::LatencyWatcher::new(),
            )),
            "cost" => registry.register_observer(Arc::new(
                forge_observers::cost_watcher::CostWatcher::new(),
            )),
            "accuracy" => registry.register_observer(Arc::new(
                forge_observers::accuracy_watcher::AccuracyWatcher::new(),
            )),
            "security" => registry.register_observer(Arc::new(
                forge_observers::security_watcher::SecurityWatcher::new(),
            )),
            "reliability" => registry.register_observer(Arc::new(
                forge_observers::reliability_watcher::ReliabilityWatcher::new(),
            )),
            "context_quality" => registry.register_observer(Arc::new(
                forge_observers::context_quality_watcher::ContextQualityWatcher::new(),
            )),
            "orch" => registry.register_observer(Arc::new(
                forge_observers::orch_watcher::OrchWatcher::new(),
            )),
            "comm" => registry.register_observer(Arc::new(
                forge_observers::comm_watcher::CommWatcher::new(),
            )),
            "compliance" => registry.register_observer(Arc::new(
                forge_observers::compliance_watcher::ComplianceWatcher::new(),
            )),
            "memory" => registry.register_observer(Arc::new(
                forge_observers::memory_watcher::MemoryWatcher::new(),
            )),
            "diversity" => registry.register_observer(Arc::new(
                forge_observers::diversity_watcher::DiversityWatcher::new(),
            )),
            other => tracing::warn!("Unknown observer '{}' — skipping", other),
        }
    }
}

// ─── Detectors (16 detectors) ─────────────────────────────────────────
// Implement forge_sdk::traits::detector::Detector
// NOTE: some are unit structs (no new()), others have parameterized new()

fn register_detectors(registry: &mut PluginRegistry, names: &[&str]) {
    for name in names {
        match *name {
            "loop" => registry.register_detector(Arc::new(
                forge_detectors::loop_detector::LoopDetector::new(4, 10),
            )),
            "stale_context" => registry.register_detector(Arc::new(
                forge_detectors::stale_context::StaleContextDetector::new(3, 0.85),
            )),
            "cost_anomaly" => registry.register_detector(Arc::new(
                forge_detectors::cost_anomaly::CostAnomalyDetector::new(3.0),
            )),
            "deadlock" => registry.register_detector(Arc::new(
                forge_detectors::deadlock::DeadlockDetector::new(60),
            )),
            "hallucination" => registry.register_detector(Arc::new(
                forge_detectors::hallucination::HallucinationDetector::new("."),
            )),
            "prompt_injection" => registry.register_detector(Arc::new(
                forge_detectors::prompt_injection::PromptInjectionDetector,
            )),
            "secret_leak" => registry.register_detector(Arc::new(
                forge_detectors::secret_leak::SecretLeakDetector,
            )),
            "variety_collapse" => registry.register_detector(Arc::new(
                forge_detectors::variety_collapse::VarietyCollapseDetector::new(0.85),
            )),
            "conversation_stall" => registry.register_detector(Arc::new(
                forge_detectors::conversation_stall::ConversationStallDetector::new(45),
            )),
            "goal_drift" => registry.register_detector(Arc::new(
                forge_detectors::goal_drift::GoalDriftDetector::new(0.4),
            )),
            "model_mismatch" => registry.register_detector(Arc::new(
                forge_detectors::model_mismatch::ModelMismatchDetector,
            )),
            "accuracy_risk" => registry.register_detector(Arc::new(
                forge_detectors::accuracy_risk::AccuracyRiskDetector,
            )),
            "runaway_cost" => registry.register_detector(Arc::new(
                forge_detectors::runaway_cost::RunawayCostDetector::new(1.2),
            )),
            "resource_exhaustion" => registry.register_detector(Arc::new(
                forge_detectors::resource_exhaustion::ResourceExhaustionDetector::new(80.0, 90.0),
            )),
            "output_degradation" => registry.register_detector(Arc::new(
                forge_detectors::output_degradation::OutputDegradationDetector::new(0.3),
            )),
            "compliance_gap" => registry.register_detector(Arc::new(
                forge_detectors::compliance_gap::ComplianceGapDetector,
            )),
            other => tracing::warn!("Unknown detector '{}' — skipping", other),
        }
    }
}

// ─── Strategies (14 strategies) ───────────────────────────────────────
// Implement forge_sdk::traits::strategy::Strategy
// Most are unit structs; only NudgeStrategy & CompactStrategy take params

fn register_strategies(registry: &mut PluginRegistry, names: &[&str]) {
    for name in names {
        match *name {
            "nudge" => registry.register_strategy(Arc::new(
                forge_strategies::nudge::NudgeStrategy::new(3),
            )),
            "compact" => registry.register_strategy(Arc::new(
                forge_strategies::compact::CompactStrategy::new(0.6),
            )),
            "pause" => registry.register_strategy(Arc::new(
                forge_strategies::pause::PauseStrategy,
            )),
            "escalate" => registry.register_strategy(Arc::new(
                forge_strategies::escalate::EscalateStrategy,
            )),
            "fork" => registry.register_strategy(Arc::new(
                forge_strategies::fork::ForkStrategy,
            )),
            "reroute" => registry.register_strategy(Arc::new(
                forge_strategies::reroute::RerouteStrategy,
            )),
            "rollback" => registry.register_strategy(Arc::new(
                forge_strategies::rollback::RollbackStrategy,
            )),
            "diversify" => registry.register_strategy(Arc::new(
                forge_strategies::diversify::DiversifyStrategy,
            )),
            "isolate" => registry.register_strategy(Arc::new(
                forge_strategies::isolate::IsolateStrategy,
            )),
            "circuit_break" => registry.register_strategy(Arc::new(
                forge_strategies::circuit_break::CircuitBreakStrategy,
            )),
            "replace" => registry.register_strategy(Arc::new(
                forge_strategies::replace::ReplaceStrategy,
            )),
            "interject" => registry.register_strategy(Arc::new(
                forge_strategies::interject::InterjectStrategy,
            )),
            "degrade" => registry.register_strategy(Arc::new(
                forge_strategies::degrade::DegradeStrategy,
            )),
            "quarantine" => registry.register_strategy(Arc::new(
                forge_strategies::quarantine::QuarantineStrategy,
            )),
            other => tracing::warn!("Unknown strategy '{}' — skipping", other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_solo_registry() {
        let registry = build_registry_from_preset(&Preset::Solo);
        assert!(!registry.observers().is_empty());
        assert!(!registry.detectors().is_empty());
        assert!(!registry.strategies().is_empty());
    }

    #[test]
    fn test_build_claude_code_registry() {
        let registry = build_registry_from_preset(&Preset::ClaudeCode);
        assert!(registry.observers().len() >= 5);
        assert!(registry.detectors().len() >= 6);
        assert!(registry.strategies().len() >= 6);
    }

    #[test]
    fn test_build_cloud_agent_registry() {
        let registry = build_registry_from_preset(&Preset::Devin);
        assert!(registry.observers().len() >= 3);
        assert!(registry.detectors().len() >= 2);
        assert!(!registry.strategies().is_empty());
    }

    #[test]
    fn test_all_31_presets_build_without_panic() {
        let presets = [
            Preset::Solo, Preset::LangGraph, Preset::CrewAI, Preset::AutoGen,
            Preset::LangChain, Preset::OpenAISwarm, Preset::SemanticKernel,
            Preset::Haystack, Preset::DSPy, Preset::LlamaIndex, Preset::TaskWeaver,
            Preset::Agno, Preset::AtomicAgents, Preset::BeeAgent, Preset::PydanticAI,
            Preset::ClaudeCode, Preset::Aider, Preset::Cline, Preset::Continue,
            Preset::VercelAI, Preset::Copilot, Preset::Cursor, Preset::Windsurf,
            Preset::Devin, Preset::AmazonQ, Preset::ReplitAgent, Preset::PearAI,
            Preset::BoltNew, Preset::Lovable, Preset::V0, Preset::Custom,
        ];
        for preset in &presets {
            let registry = build_registry_from_preset(preset);
            assert!(
                !registry.observers().is_empty(),
                "Preset {:?} has zero observers — factory missing observer for this preset",
                preset
            );
        }
    }

    #[test]
    fn test_unknown_names_gracefully_skipped() {
        let registry = build_registry_from_names(
            &["nonexistent"],
            &["also_fake"],
            &["made_up"],
        );
        assert!(registry.observers().is_empty());
        assert!(registry.detectors().is_empty());
        assert!(registry.strategies().is_empty());
    }
}
