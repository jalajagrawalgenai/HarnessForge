// forge-harness/src/checkpoint.rs — State snapshot and restore

use chrono::Utc;
use forge_sdk::types::audit::Checkpoint;
use serde_json::Value;
use uuid::Uuid;

/// Manages checkpoint creation and restoration for agent sessions
#[derive(Default)]
pub struct CheckpointManager {
    checkpoints: Vec<Checkpoint>,
    max_checkpoints: usize,
}

impl CheckpointManager {
    pub fn new(max_checkpoints: usize) -> Self {
        Self {
            checkpoints: Vec::new(),
            max_checkpoints,
        }
    }

    /// Save a checkpoint of the current agent state
    pub fn save(
        &mut self,
        session_id: Uuid,
        event_id: i64,
        agent_state: Value,
        token_usage: Value,
        context: Option<Value>,
        messages: Option<Value>,
    ) -> Checkpoint {
        let checkpoint = Checkpoint {
            id: Uuid::new_v4(),
            event_id,
            session_id,
            agent_states: agent_state,
            context_snapshot: context,
            message_queue: messages,
            state_store: None,
            graph_state: None,
            task_progress: None,
            plan: None,
            token_usage,
            created_at: Utc::now(),
        };

        self.checkpoints.push(checkpoint.clone());

        // Evict oldest if over max
        if self.checkpoints.len() > self.max_checkpoints {
            self.checkpoints.remove(0);
        }

        checkpoint
    }

    /// Get the most recent checkpoint
    pub fn latest(&self) -> Option<&Checkpoint> {
        self.checkpoints.last()
    }

    /// Get a specific checkpoint by ID
    pub fn get(&self, id: &Uuid) -> Option<&Checkpoint> {
        self.checkpoints.iter().find(|c| &c.id == id)
    }

    /// Get the checkpoint nearest to (but before) an event
    pub fn nearest_before(&self, event_id: i64) -> Option<&Checkpoint> {
        self.checkpoints
            .iter()
            .rev()
            .find(|c| c.event_id <= event_id)
    }

    /// List all checkpoints for this session
    pub fn list(&self) -> &[Checkpoint] {
        &self.checkpoints
    }
}
