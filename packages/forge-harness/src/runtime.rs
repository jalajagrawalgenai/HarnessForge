// forge-harness/src/runtime.rs — Spawns agent + harness, manages lifecycle

use forge_sdk::agent::AgentAdapter;
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention};
use tokio::sync::mpsc;

use crate::pipeline::Pipeline;

/// The runtime manages a single agent session with the harness
pub struct Runtime {
    session_id: String,
    pipeline: Pipeline,
    agent_event_rx: Option<mpsc::Receiver<AgentEvent>>,
    intervention_tx: Option<mpsc::Sender<Intervention>>,
}

impl Runtime {
    pub fn new(session_id: String, pipeline: Pipeline) -> Self {
        Self {
            session_id,
            pipeline,
            agent_event_rx: None,
            intervention_tx: None,
        }
    }

    /// Run an agent inside the harness
    pub async fn run(
        &mut self,
        agent: &mut dyn AgentAdapter,
        task: &str,
    ) -> Result<SessionResult, ForgeError> {
        let (_event_tx, event_rx) = mpsc::channel::<AgentEvent>(256);
        let (intervention_tx, _intervention_rx) = mpsc::channel::<Intervention>(64);

        self.agent_event_rx = Some(event_rx);
        self.intervention_tx = Some(intervention_tx.clone());

        let agent_id = agent.id();
        let agent_type = agent.agent_type();

        // Emit started event
        let started = AgentEvent::Started {
            agent_id: agent_id.clone(),
            task: task.to_string(),
            timestamp: chrono::Utc::now(),
        };
        self.pipeline.feed(&started);

        // Spawn the agent (in a real impl, this runs concurrently with harness)
        let mut outcome = AgentOutcome {
            success: false,
            summary: String::new(),
            output: None,
        };

        // Process agent events in a loop
        // In a real implementation, this runs the agent and harness concurrently
        // For now, we simulate the cycle
        let turn = 0u32;
        let _max_turns = 100;

        // Emit thinking/completion based on task
        self.pipeline.feed(&AgentEvent::ThinkingStart {
            agent_id: agent_id.clone(),
            timestamp: chrono::Utc::now(),
        });

        self.pipeline.feed(&AgentEvent::OutputComplete {
            agent_id: agent_id.clone(),
            content: format!("Task processed: {}", task),
            timestamp: chrono::Utc::now(),
        });

        // Run pipeline cycle
        let interventions = self.pipeline.cycle(&agent_id).await;

        // Apply interventions
        for intervention in &interventions {
            let _ = intervention_tx.send(intervention.clone()).await;
        }

        self.pipeline.feed(&AgentEvent::Completed {
            agent_id: agent_id.clone(),
            summary: "Task completed".to_string(),
            timestamp: chrono::Utc::now(),
        });

        outcome.success = true;
        outcome.summary = "Task completed successfully".to_string();

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

    /// Pause the session (human gate)
    pub async fn pause(&mut self) -> Result<(), ForgeError> {
        if let Some(ref tx) = self.intervention_tx {
            let intervention = Intervention::Pause {
                reason: "Human gate triggered".to_string(),
                checkpoint_id: uuid::Uuid::new_v4(),
            };
            tx.send(intervention)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?;
        }
        Ok(())
    }

    /// Resume the session
    pub async fn resume(&mut self) -> Result<(), ForgeError> {
        if let Some(ref tx) = self.intervention_tx {
            tx.send(Intervention::Resume)
                .await
                .map_err(|e| ForgeError::Internal(e.to_string()))?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SessionResult {
    pub session_id: String,
    pub agent_id: String,
    pub agent_type: forge_sdk::agent::AgentType,
    pub outcome: AgentOutcome,
    pub turns: u32,
    pub pipeline_stats: crate::pipeline::PipelineStats,
}
