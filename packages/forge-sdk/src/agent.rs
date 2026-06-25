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
    // ─── General-purpose ───
    Solo,
    Custom,

    // ─── Agent frameworks ───
    LangGraph,
    CrewAI,
    AutoGen,
    LangChain,
    OpenAISwarm,
    SemanticKernel,
    Haystack,
    DSPy,
    LlamaIndex,
    TaskWeaver,
    Agno,
    AtomicAgents,
    BeeAgent,
    PydanticAI,

    // ─── Coding agents / CLI agents ───
    ClaudeCode,
    Aider,
    Cline,
    Continue,

    // ─── Cloud / IDE agents (API/webhook bridged) ───
    VercelAI,
    Copilot,
    Cursor,
    Windsurf,
    Devin,
    AmazonQ,
    ReplitAgent,
    PearAI,
    BoltNew,
    Lovable,
    V0,
}

impl AgentType {
    /// Is this agent primarily Python-based?
    pub fn is_python(&self) -> bool {
        matches!(
            self,
            AgentType::LangGraph
                | AgentType::CrewAI
                | AgentType::AutoGen
                | AgentType::LangChain
                | AgentType::OpenAISwarm
                | AgentType::SemanticKernel
                | AgentType::Haystack
                | AgentType::DSPy
                | AgentType::LlamaIndex
                | AgentType::TaskWeaver
                | AgentType::Agno
                | AgentType::AtomicAgents
                | AgentType::PydanticAI
                | AgentType::Aider
        )
    }

    /// Is this agent TypeScript/Node-based?
    pub fn is_typescript(&self) -> bool {
        matches!(
            self,
            AgentType::ClaudeCode | AgentType::Cline | AgentType::Continue | AgentType::VercelAI
        )
    }

    /// Is this agent an IDE extension or cloud service (API bridged)?
    pub fn is_bridged(&self) -> bool {
        matches!(
            self,
            AgentType::Copilot
                | AgentType::Cursor
                | AgentType::Windsurf
                | AgentType::Devin
                | AgentType::AmazonQ
                | AgentType::ReplitAgent
                | AgentType::PearAI
                | AgentType::BoltNew
                | AgentType::Lovable
                | AgentType::V0
        )
    }

    /// Does this agent support event streaming natively?
    pub fn supports_streaming(&self) -> bool {
        !self.is_bridged()
    }

    /// Bridge method — how to connect this agent type to Forge
    pub fn bridge_method(&self) -> BridgeMethod {
        match self {
            AgentType::Solo | AgentType::Custom => BridgeMethod::RustNative,
            _ if self.is_python() => BridgeMethod::PyO3,
            _ if self.is_typescript() => BridgeMethod::NAPI,
            _ if self.is_bridged() => BridgeMethod::RestOrMCP,
            AgentType::BeeAgent => BridgeMethod::PyO3, // Primarily Python
            _ => BridgeMethod::RestOrMCP,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BridgeMethod {
    /// Rust-native AgentAdapter trait impl
    RustNative,
    /// PyO3 bridge (Python agent → Rust harness)
    PyO3,
    /// NAPI-RS bridge (TypeScript agent → Rust harness)
    NAPI,
    /// REST API or MCP bridge (IDE/cloud agent → harness)
    RestOrMCP,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AgentType::Solo => "solo",
            AgentType::Custom => "custom",
            AgentType::LangGraph => "langgraph",
            AgentType::CrewAI => "crewai",
            AgentType::AutoGen => "autogen",
            AgentType::LangChain => "langchain",
            AgentType::OpenAISwarm => "openai-swarm",
            AgentType::SemanticKernel => "semantic-kernel",
            AgentType::Haystack => "haystack",
            AgentType::DSPy => "dspy",
            AgentType::LlamaIndex => "llamaindex",
            AgentType::TaskWeaver => "taskweaver",
            AgentType::Agno => "agno",
            AgentType::AtomicAgents => "atomic-agents",
            AgentType::BeeAgent => "bee-agent",
            AgentType::PydanticAI => "pydantic-ai",
            AgentType::ClaudeCode => "claude-code",
            AgentType::Aider => "aider",
            AgentType::Cline => "cline",
            AgentType::Continue => "continue",
            AgentType::VercelAI => "vercel-ai",
            AgentType::Copilot => "copilot",
            AgentType::Cursor => "cursor",
            AgentType::Windsurf => "windsurf",
            AgentType::Devin => "devin",
            AgentType::AmazonQ => "amazon-q",
            AgentType::ReplitAgent => "replit-agent",
            AgentType::PearAI => "pearai",
            AgentType::BoltNew => "bolt-new",
            AgentType::Lovable => "lovable",
            AgentType::V0 => "v0",
        };
        write!(f, "{}", s)
    }
}

// ─── MockAgentAdapter (for testing) ───────────────────────────────────

/// A minimal agent that simulates a few turns of work.
/// Used for integration tests and as a starting template for real adapters.
pub struct MockAgent {
    pub id: String,
    pub agent_type: AgentType,
    pub turn_count: u32,
    pub succeed: bool,
}

impl MockAgent {
    pub fn new(id: impl Into<String>, agent_type: AgentType) -> Self {
        Self { id: id.into(), agent_type, turn_count: 3, succeed: true }
    }

    pub fn with_turns(mut self, turns: u32) -> Self {
        self.turn_count = turns;
        self
    }

    pub fn with_success(mut self, succeed: bool) -> Self {
        self.succeed = succeed;
        self
    }
}

#[async_trait]
impl AgentAdapter for MockAgent {
    fn id(&self) -> String { self.id.clone() }
    fn agent_type(&self) -> AgentType { self.agent_type }

    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        _intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError> {
        use chrono::Utc;
        let now = Utc::now();

        // Emit started
        let _ = event_tx.send(AgentEvent::Started {
            agent_id: self.id.clone(),
            task: task.to_string(),
            timestamp: now,
        }).await;

        // Simulate N turns of "think + tool use"
        for turn in 0..self.turn_count {
            let _ = event_tx.send(AgentEvent::ThinkingStart {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            }).await;

            let _ = event_tx.send(AgentEvent::ToolCallStart {
                agent_id: self.id.clone(),
                tool: "mock_search".into(),
                args: serde_json::json!({"query": format!("turn_{}", turn)}),
                timestamp: Utc::now(),
            }).await;

            let _ = event_tx.send(AgentEvent::ToolCallEnd {
                agent_id: self.id.clone(),
                tool: "mock_search".into(),
                result: ToolResult {
                    content: format!("result_turn_{}", turn),
                    is_error: false,
                    duration_ms: 50,
                    token_count: 100,
                },
                timestamp: Utc::now(),
            }).await;

            let _ = event_tx.send(AgentEvent::ThinkingEnd {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            }).await;
        }

        // Emit completed
        let _ = event_tx.send(AgentEvent::Completed {
            agent_id: self.id.clone(),
            summary: format!("Mock agent completed task: {}", task),
            timestamp: Utc::now(),
        }).await;

        Ok(AgentOutcome {
            success: self.succeed,
            summary: format!("Completed {} mock turns for: {}", self.turn_count, task),
            output: Some(format!("mock_output: {} turns", self.turn_count)),
        })
    }
}

use crate::events::ToolResult;
