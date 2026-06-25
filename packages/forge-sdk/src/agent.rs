use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::error::ForgeError;
use crate::events::{AgentEvent, AgentOutcome, Intervention};

/// Implement this trait for any existing agent to run it inside the Forge harness.
#[async_trait]
pub trait AgentAdapter: Send + Sync {
    /// Unique ID for this agent instance
    fn id(&self) -> String;

    /// What kind of agent? Helps harness choose default plugins.
    fn agent_type(&self) -> AgentType;

    /// Run the agent. Forge calls this once per task.
    /// - Send events to event_tx as the agent operates
    /// - Poll intervention_rx for harness interventions between turns
    /// - Return AgentOutcome when the task is complete
    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Solo,
    LangGraph,
    CrewAI,
    AutoGen,
    Custom,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Solo => write!(f, "solo"),
            AgentType::LangGraph => write!(f, "langgraph"),
            AgentType::CrewAI => write!(f, "crewai"),
            AgentType::AutoGen => write!(f, "autogen"),
            AgentType::Custom => write!(f, "custom"),
        }
    }
}
