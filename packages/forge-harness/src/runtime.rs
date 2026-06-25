// forge-harness/src/runtime.rs — Agent + harness pipeline lifecycle
//
// The runtime orchestrates a single agent session:
// 1. Creates channels for agent↔harness communication
// 2. Awaits agent execution (agent sends events via channel)
// 3. Drains all events and feeds through the observe→detect→strategy pipeline
// 4. Runs final pipeline cycle and returns results

use forge_sdk::agent::AgentAdapter;
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention};
use forge_sdk::harness::HarnessRunResult;
use tokio::sync::mpsc;

use crate::pipeline::Pipeline;

/// Orchestrates a single agent session with the harness pipeline.
pub struct Runtime {
    session_id: String,
    pipeline: Pipeline,
}

impl Runtime {
    pub fn new(session_id: String, pipeline: Pipeline) -> Self {
        Self { session_id, pipeline }
    }

    /// Run an agent inside the harness pipeline.
    ///
    /// Design: The agent is the active party — it calls tools, thinks,
    /// and sends events to the channel. The harness observes these events
    /// and runs periodic pipeline cycles. After the agent completes, we
    /// drain remaining events and run a final cycle.
    ///
    /// For real agents (not MockAgent), the AgentAdapter impl should:
    /// - Send lifecycle/reasoning/tool events via event_tx
    /// - Periodically check intervention_rx between turns
    /// - Return AgentOutcome when the task completes
    pub async fn run(
        &mut self,
        agent: &mut (dyn AgentAdapter + Send),
        task: &str,
    ) -> Result<SessionResult, ForgeError> {
        let agent_id = agent.id();
        let agent_type = agent.agent_type();
        let task_owned = task.to_string();

        // Create channels for agent↔harness communication
        // Buffer of 1024 is generous enough for any realistic agent turn count
        let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(1024);
        let (intervention_tx, _intervention_rx) = mpsc::channel::<Intervention>(64);

        // Feed the initial Started event to the pipeline bus
        self.pipeline.feed(&AgentEvent::Started {
            agent_id: agent_id.clone(),
            task: task.to_string(),
            timestamp: chrono::Utc::now(),
        });

        // Run the agent. It sends events to event_tx during execution.
        // The intervention_rx is passed to the agent for it to poll interventions.
        let outcome = agent
            .run(&task_owned, event_tx, _intervention_rx)
            .await?;

        // Drain all events the agent emitted during execution
        let mut turn = 0u32;
        let mut last_intervention_cycle = 0u32;

        while let Ok(event) = event_rx.try_recv() {
            let is_completion = matches!(
                &event,
                AgentEvent::Completed { .. } | AgentEvent::Failed { .. }
            );

            self.pipeline.feed(&event);

            if is_completion {
                continue;
            }

            turn += 1;

            // Run pipeline cycle periodically (every 3 turns or after tool calls)
            let should_cycle = turn.is_multiple_of(3)
                || matches!(&event, AgentEvent::ToolCallEnd { .. });
            if should_cycle && turn > last_intervention_cycle {
                let interventions = self.pipeline.cycle(&agent_id).await;
                for iv in &interventions {
                    // Best-effort send — agent may have already completed
                    let _ = intervention_tx.try_send(iv.clone());
                }
                last_intervention_cycle = turn;
            }
        }

        // Run a final pipeline cycle to pick up any remaining patterns
        let _ = self.pipeline.cycle(&agent_id).await;

        let stats = self.pipeline.stats();

        Ok(SessionResult {
            session_id: self.session_id.clone(),
            agent_id,
            agent_type,
            outcome,
            turns: turn,
            pipeline_stats: stats,
        })
    }
}

// ─── SessionResult ────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct SessionResult {
    pub session_id: String,
    pub agent_id: String,
    pub agent_type: forge_sdk::agent::AgentType,
    pub outcome: AgentOutcome,
    pub turns: u32,
    pub pipeline_stats: crate::pipeline::PipelineStats,
}

impl SessionResult {
    /// Convert to the SDK's public HarnessRunResult.
    pub fn into_harness_result(self) -> HarnessRunResult {
        HarnessRunResult {
            agent_id: self.agent_id,
            observation_count: self.pipeline_stats.cycles as u64,
            detection_count: self.pipeline_stats.total_detections as u64,
            intervention_count: self.pipeline_stats.total_interventions as u64,
            success: self.outcome.success,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use forge_sdk::agent::{AgentType, MockAgent};
    use crate::factory::build_registry_from_preset;
    use forge_sdk::presets::Preset;

    #[tokio::test]
    async fn test_runtime_runs_mock_agent() {
        let registry = build_registry_from_preset(&Preset::Solo);
        let pipeline = Pipeline::new(Arc::new(registry), false);
        let mut runtime = Runtime::new("test-session".into(), pipeline);
        let mut agent = MockAgent::new("test-agent", AgentType::Solo);

        let result = runtime.run(&mut agent, "hello world").await;
        assert!(result.is_ok());
        let session = result.unwrap();
        assert_eq!(session.agent_id, "test-agent");
        assert!(session.outcome.success);
        assert!(session.turns > 0);
        assert!(session.pipeline_stats.cycles > 0);
    }

    #[tokio::test]
    async fn test_runtime_with_claude_code_preset() {
        let registry = build_registry_from_preset(&Preset::ClaudeCode);
        let pipeline = Pipeline::new(Arc::new(registry), false);
        let mut runtime = Runtime::new("cc-session".into(), pipeline);
        let mut agent = MockAgent::new("cc-agent", AgentType::ClaudeCode).with_turns(5);

        let result = runtime.run(&mut agent, "refactor auth module").await;
        assert!(result.is_ok());
        let session = result.unwrap();
        assert!(session.outcome.success);
    }
}
