//! AdapterFactory — Creates the right AgentAdapter for any AgentType.
//!
//! ```rust
//! use forge_adapters::AdapterFactory;
//! use forge_sdk::agent::AgentType;
//!
//! // Automatically gets the right adapter:
//! let claude = AdapterFactory::create(AgentType::ClaudeCode, "id", None);
//! let langgraph = AdapterFactory::create(AgentType::LangGraph, "id", None);
//! let copilot = AdapterFactory::create(AgentType::Copilot, "id", None);
//! ```

use crate::{BridgeAgent, CliAgent, HttpAgent, PythonAgent};
use forge_sdk::agent::{AgentAdapter, AgentType};

/// Optional configuration for the factory.
#[derive(Default)]
pub struct FactoryConfig {
    /// API key for HTTP-based agents
    pub api_key: Option<String>,
    /// Base URL for HTTP-based agents
    pub base_url: Option<String>,
    /// Model name for HTTP-based agents
    pub model: Option<String>,
    /// Working directory for CLI/Python agents
    pub work_dir: Option<String>,
    /// Python path for Python-based agents
    pub python_path: Option<String>,
    /// Custom CLI command override
    pub cli_command: Option<String>,
    /// Custom Python script for Python agents
    pub python_script: Option<String>,
    /// Bridge callback URL
    pub callback_url: Option<String>,
    /// Bridge MCP endpoint
    pub mcp_endpoint: Option<String>,
}

/// Creates the appropriate AgentAdapter implementation for any agent type.
pub struct AdapterFactory;

impl AdapterFactory {
    /// Create an AgentAdapter for the given agent type.
    ///
    /// The factory automatically selects the right adapter:
    /// - CLI agents (Claude Code, Aider, Cline, Continue) → `CliAgent`
    /// - HTTP API agents (Solo, BeeAgent, PydanticAI) → `HttpAgent`
    /// - Python framework agents (LangGraph, CrewAI, etc.) → `PythonAgent`
    /// - IDE/Cloud bridged agents (Copilot, Cursor, etc.) → `BridgeAgent`
    /// - Custom → `CliAgent` as flexible default
    pub fn create(
        agent_type: AgentType,
        id: impl Into<String>,
        config: Option<FactoryConfig>,
    ) -> Box<dyn AgentAdapter> {
        let config = config.unwrap_or_default();
        let id = id.into();

        match agent_type {
            // ─── CLI-based agents ───
            AgentType::ClaudeCode | AgentType::Aider | AgentType::Cline | AgentType::Continue => {
                let mut agent = CliAgent::new(id, agent_type);
                if let Some(ref cmd) = config.cli_command {
                    agent = agent.command(cmd.clone());
                }
                if let Some(ref dir) = config.work_dir {
                    agent = agent.work_dir(dir.clone());
                }
                Box::new(agent)
            }

            // ─── HTTP API agents ───
            AgentType::Solo
            | AgentType::PydanticAI
            | AgentType::BeeAgent
            | AgentType::Custom => {
                let mut agent = HttpAgent::new(id, agent_type);
                if let Some(ref key) = config.api_key {
                    agent = agent.api_key(key.clone());
                }
                if let Some(ref url) = config.base_url {
                    agent = agent.base_url(url.clone());
                }
                if let Some(ref model) = config.model {
                    agent = agent.model(model.clone());
                }
                Box::new(agent)
            }

            // ─── Python framework agents ───
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
            | AgentType::AtomicAgents => {
                let mut agent = PythonAgent::new(id, agent_type);
                if let Some(ref script) = config.python_script {
                    agent = agent.script(script.clone());
                }
                if let Some(ref py) = config.python_path {
                    agent = agent.python_path(py.clone());
                }
                if let Some(ref dir) = config.work_dir {
                    agent = agent.work_dir(dir.clone());
                }
                Box::new(agent)
            }

            // ─── IDE / Cloud bridged agents ───
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
            | AgentType::VercelAI => {
                let mut agent = BridgeAgent::new(id, agent_type);
                if let Some(ref url) = config.callback_url {
                    agent = agent.callback_url(url.clone());
                }
                if let Some(ref url) = config.mcp_endpoint {
                    agent = agent.mcp_endpoint(url.clone());
                }
                if let Some(ref key) = config.api_key {
                    agent = agent.api_key(key.clone());
                }
                Box::new(agent)
            }
        }
    }

    /// Return which adapter type a given AgentType maps to (for display/info).
    pub fn adapter_name(agent_type: AgentType) -> &'static str {
        match agent_type {
            AgentType::ClaudeCode | AgentType::Aider | AgentType::Cline | AgentType::Continue => {
                "CliAgent (subprocess)"
            }
            AgentType::Solo
            | AgentType::PydanticAI
            | AgentType::BeeAgent
            | AgentType::Custom => "HttpAgent (API)",
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
            | AgentType::AtomicAgents => "PythonAgent (subprocess)",
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
            | AgentType::VercelAI => "BridgeAgent (REST/MCP)",
        }
    }

    /// List all agent types with their adapter mappings.
    pub fn list_all() -> Vec<(AgentType, &'static str)> {
        use AgentType::*;
        let all = [
            // CLI
            ClaudeCode, Aider, Cline, Continue,
            // HTTP API
            Solo, BeeAgent, PydanticAI, Custom,
            // Python
            LangGraph, CrewAI, AutoGen, LangChain, OpenAISwarm, SemanticKernel,
            Haystack, DSPy, LlamaIndex, TaskWeaver, Agno, AtomicAgents,
            // Bridge
            Copilot, Cursor, Windsurf, Devin, AmazonQ, ReplitAgent,
            PearAI, BoltNew, Lovable, V0, VercelAI,
        ];
        all.into_iter()
            .map(|at| (at, Self::adapter_name(at)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_covers_all_31_types() {
        let all = AdapterFactory::list_all();
        assert_eq!(all.len(), 31, "All 31 agent types must be covered");
    }

    #[test]
    fn test_factory_creates_every_type() {
        for (agent_type, _name) in AdapterFactory::list_all() {
            let adapter = AdapterFactory::create(agent_type, "test", None);
            assert_eq!(adapter.agent_type(), agent_type);
            assert!(!adapter.id().is_empty());
        }
    }

    #[test]
    fn test_factory_cli_agents() {
        let adapter = AdapterFactory::create(AgentType::ClaudeCode, "claude-1", None);
        assert_eq!(adapter.agent_type(), AgentType::ClaudeCode);

        let adapter = AdapterFactory::create(AgentType::Aider, "aider-1", None);
        assert_eq!(adapter.agent_type(), AgentType::Aider);
    }

    #[test]
    fn test_factory_http_agents() {
        let adapter = AdapterFactory::create(
            AgentType::Solo,
            "solo-1",
            Some(FactoryConfig {
                api_key: Some("test-key".into()),
                model: Some("claude-sonnet-4-6".into()),
                ..Default::default()
            }),
        );
        assert_eq!(adapter.agent_type(), AgentType::Solo);
    }

    #[test]
    fn test_factory_python_agents() {
        let adapter = AdapterFactory::create(AgentType::LangGraph, "lg-1", None);
        assert_eq!(adapter.agent_type(), AgentType::LangGraph);

        let adapter = AdapterFactory::create(AgentType::CrewAI, "crew-1", None);
        assert_eq!(adapter.agent_type(), AgentType::CrewAI);
    }

    #[test]
    fn test_factory_bridge_agents() {
        let adapter = AdapterFactory::create(AgentType::Copilot, "copilot-1", None);
        assert_eq!(adapter.agent_type(), AgentType::Copilot);

        let adapter = AdapterFactory::create(AgentType::Devin, "devin-1", None);
        assert_eq!(adapter.agent_type(), AgentType::Devin);
    }

    #[test]
    fn test_adapter_names_distinct() {
        let names: std::collections::HashSet<&str> = AdapterFactory::list_all()
            .into_iter()
            .map(|(_, name)| name)
            .collect();
        assert_eq!(names.len(), 4); // 4 adapter types
    }
}
