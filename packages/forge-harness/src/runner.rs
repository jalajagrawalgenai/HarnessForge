// forge-harness/src/runner.rs — Session orchestrator
//
// THE entry point. Ties Harness + Agent + Pipeline + Runtime together.
// CLI and server both call `run_harness_session()`.

use std::sync::Arc;
use async_trait::async_trait;
use forge_sdk::agent::AgentAdapter;
use forge_sdk::error::ForgeError;
use forge_sdk::harness::{Harness, HarnessConfig, HarnessRuntime, HarnessRunResult};
use forge_sdk::presets::Preset;
use forge_sdk::traits::store::AuditStore;

use crate::factory::build_registry_from_preset;
use crate::pipeline::Pipeline;
use crate::plugin_registry::PluginRegistry;
use crate::runtime::{Runtime, SessionResult};

// ─── Free functions (primary API) ─────────────────────────────────────

/// Run an agent session under a preset. Most common entry point.
pub async fn run_harness_session(
    agent: &mut dyn AgentAdapter,
    task: &str,
    preset: Preset,
    audit_store: Option<Arc<dyn AuditStore>>,
) -> Result<HarnessRunResult, ForgeError> {
    let registry = build_registry_from_preset(&preset);
    let config = HarnessConfig::default();
    run_with_registry(agent, task, registry, config, audit_store).await
}

/// Run with a fully custom PluginRegistry (no preset needed).
pub async fn run_with_registry(
    agent: &mut dyn AgentAdapter,
    task: &str,
    registry: PluginRegistry,
    config: HarnessConfig,
    _audit_store: Option<Arc<dyn AuditStore>>,
) -> Result<HarnessRunResult, ForgeError> {
    let session_id = config
        .session_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let pipeline = Pipeline::new(Arc::new(registry), config.dry_run);
    let mut runtime = Runtime::new(session_id.clone(), pipeline);

    let result = runtime.run(agent, task).await?;

    Ok(SessionResult::into_harness_result(result))
}

/// Quick-run: Solo preset, no audit. Good for smoke tests and hello-world.
pub async fn quick_run(
    agent: &mut dyn AgentAdapter,
    task: &str,
) -> Result<HarnessRunResult, ForgeError> {
    run_harness_session(agent, task, Preset::Solo, None).await
}

/// Dry-run: observe and detect but never intervene. Used for testing configs.
pub async fn dry_run(
    agent: &mut dyn AgentAdapter,
    task: &str,
    preset: Preset,
) -> Result<HarnessRunResult, ForgeError> {
    let config = HarnessConfig { dry_run: true, ..Default::default() };
    let registry = build_registry_from_preset(&preset);
    run_with_registry(agent, task, registry, config, None).await
}

// ─── HarnessRuntime impl ──────────────────────────────────────────────

/// The default runtime backend. Implements the SDK's `HarnessRuntime` trait
/// so that `Harness::run_with()` works with forge-harness.
pub struct DefaultHarnessRuntime;

#[async_trait]
impl HarnessRuntime for DefaultHarnessRuntime {
    async fn execute(
        &self,
        _harness: &Harness,
        agent: &mut (dyn AgentAdapter + Send),
        task: &str,
    ) -> Result<HarnessRunResult, ForgeError> {
        // Build a Solo preset registry by default (harness config extension point)
        quick_run(agent, task).await
    }
}

// ─── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use forge_sdk::agent::{AgentType, MockAgent};

    #[tokio::test]
    async fn test_quick_run_succeeds() {
        let mut agent = MockAgent::new("mock-1", AgentType::Solo);
        let result = quick_run(&mut agent, "test task").await;
        assert!(result.is_ok());
        let r = result.unwrap();
        assert_eq!(r.agent_id, "mock-1");
        assert!(r.success);
    }

    #[tokio::test]
    async fn test_dry_run_no_panic() {
        let mut agent = MockAgent::new("mock-2", AgentType::Solo);
        let result = dry_run(&mut agent, "dry run task", Preset::Solo).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_run_with_claude_code_preset() {
        let mut agent = MockAgent::new("mock-cc", AgentType::ClaudeCode);
        let result = run_harness_session(&mut agent, "code review", Preset::ClaudeCode, None).await;
        assert!(result.is_ok());
        assert!(result.unwrap().success);
    }

    #[tokio::test]
    async fn test_harness_runtime_trait_works() {
        let runtime = DefaultHarnessRuntime;
        let harness = Harness::builder().build();
        let mut agent = MockAgent::new("trait-test", AgentType::Solo);
        let result = harness.run_with(&runtime, &mut agent, "trait task").await;
        assert!(result.is_ok());
    }
}
