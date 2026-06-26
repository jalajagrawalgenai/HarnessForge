//! Forge Adapters — Real AgentAdapter implementations for all 31 agent types.
//!
//! Four adapter patterns cover the entire spectrum:
//!
//! | Adapter | Covers | How |
//! |---|---|---|
//! | `CliAgent` | Claude Code, Aider, Cline, Continue | Spawns CLI tool as subprocess |
//! | `HttpAgent` | Solo, OpenAI, Anthropic, BeeAgent, PydanticAI | HTTP POST to model API |
//! | `PythonAgent` | LangGraph, CrewAI, AutoGen, LangChain, DSPy, etc. | Spawns Python script via subprocess |
//! | `BridgeAgent` | Copilot, Cursor, Windsurf, Devin, AmazonQ, etc. | REST/MCP bridge connector |
//!
//! ```rust
//! use forge_adapters::AdapterFactory;
//! use forge_sdk::agent::AgentType;
//!
//! // Create the right adapter for any agent type — automatically.
//! let mut agent = AdapterFactory::create(AgentType::ClaudeCode, "my-agent", None);
//! let mut agent = AdapterFactory::create(AgentType::LangGraph, "my-agent", None);
//! let mut agent = AdapterFactory::create(AgentType::Copilot, "my-agent", None);
//! ```

pub mod cli;
pub mod http;
pub mod python;
pub mod bridge;
pub mod factory;

pub use cli::CliAgent;
pub use http::HttpAgent;
pub use http::ApiFormat;
pub use python::PythonAgent;
pub use bridge::BridgeAgent;
pub use factory::AdapterFactory;
