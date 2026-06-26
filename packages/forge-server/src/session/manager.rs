//! Session manager — spawns and manages harness session tokio tasks.
//!
//! Each session runs as a background tokio task. The manager creates
//! the agent, pipeline, and runtime, then monitors for completion.

use crate::session::store::{SessionStatus, SharedSessionStore};
use forge_harness::factory::build_registry_from_preset;
use forge_harness::pipeline::Pipeline;
use forge_harness::runtime::Runtime;
use forge_sdk::agent::{AgentType, MockAgent};
use forge_sdk::presets::Preset;
use std::sync::Arc;
use tracing;

/// Spawn a harness session as a background tokio task.
///
/// The session will run to completion (or failure/cancellation) and
/// update the shared store with its result.
pub async fn spawn_session(
    store: SharedSessionStore,
    session_id: String,
    task: String,
    agent_type_str: String,
    preset_str: String,
) {
    // Parse agent type and preset
    let agent_type = parse_agent_type(&agent_type_str);
    let preset = parse_preset(&preset_str);

    // Create the agent
    let mut agent = MockAgent::new(&session_id, agent_type)
        .with_turns(5)
        .with_success(true);

    // Build registry and pipeline
    let registry = build_registry_from_preset(&preset);
    let pipeline = {
        let sessions = store.read().await;
        let broadcaster = sessions
            .get(&session_id)
            .map(|s| s.event_broadcaster.clone());
        match broadcaster {
            Some(tx) => Pipeline::new(Arc::new(registry), false)
                .with_event_broadcaster(tx),
            None => Pipeline::new(Arc::new(registry), false),
        }
    };

    let mut runtime = Runtime::new(session_id.clone(), pipeline);

    // Mark running
    {
        let mut sessions = store.write().await;
        if let Some(s) = sessions.get_mut(&session_id) {
            s.status = SessionStatus::Running;
        }
    }

    tracing::info!(
        session_id = %session_id,
        task = %task,
        agent_type = %agent_type_str,
        preset = %preset_str,
        "Starting harness session"
    );

    // Run the harness
    let result = runtime.run(&mut agent, &task).await;

    // Update store with result
    {
        let mut sessions = store.write().await;
        if let Some(s) = sessions.get_mut(&session_id) {
            match result {
                Ok(session_result) => {
                    s.status = if session_result.outcome.success {
                        SessionStatus::Completed
                    } else {
                        SessionStatus::Failed
                    };
                    s.result = Some(forge_sdk::harness::HarnessRunResult {
                        agent_id: session_result.agent_id,
                        observation_count: session_result.pipeline_stats.cycles as u64,
                        detection_count: session_result.pipeline_stats.total_detections
                            as u64,
                        intervention_count: session_result
                            .pipeline_stats
                            .total_interventions as u64,
                        success: session_result.outcome.success,
                    });
                }
                Err(e) => {
                    s.status = SessionStatus::Failed;
                    tracing::error!(
                        session_id = %session_id,
                        error = %e,
                        "Harness session failed"
                    );
                }
            }
            s.completed_at = Some(chrono::Utc::now());
        }
    }

    tracing::info!(
        session_id = %session_id,
        "Harness session completed"
    );
}

fn parse_agent_type(s: &str) -> AgentType {
    match s.to_lowercase().as_str() {
        "solo" => AgentType::Solo,
        "custom" => AgentType::Custom,
        "langgraph" => AgentType::LangGraph,
        "crewai" | "crew_ai" => AgentType::CrewAI,
        "autogen" => AgentType::AutoGen,
        "langchain" => AgentType::LangChain,
        "openai-swarm" | "openaiswarm" | "openai_swarm" => AgentType::OpenAISwarm,
        "semantic-kernel" | "semantickernel" => AgentType::SemanticKernel,
        "haystack" => AgentType::Haystack,
        "dspy" => AgentType::DSPy,
        "llamaindex" | "llama-index" => AgentType::LlamaIndex,
        "taskweaver" => AgentType::TaskWeaver,
        "agno" => AgentType::Agno,
        "atomic-agents" | "atomicagents" => AgentType::AtomicAgents,
        "bee-agent" | "beeagent" => AgentType::BeeAgent,
        "pydantic-ai" | "pydanticai" => AgentType::PydanticAI,
        "claude-code" | "claudecode" => AgentType::ClaudeCode,
        "aider" => AgentType::Aider,
        "cline" => AgentType::Cline,
        "continue" => AgentType::Continue,
        "vercel-ai" | "vercelai" => AgentType::VercelAI,
        "copilot" => AgentType::Copilot,
        "cursor" => AgentType::Cursor,
        "windsurf" => AgentType::Windsurf,
        "devin" => AgentType::Devin,
        "amazon-q" | "amazonq" => AgentType::AmazonQ,
        "replit-agent" | "replitagent" => AgentType::ReplitAgent,
        "pearai" | "pear-ai" => AgentType::PearAI,
        "bolt-new" | "boltnew" => AgentType::BoltNew,
        "lovable" => AgentType::Lovable,
        "v0" => AgentType::V0,
        _ => AgentType::Solo,
    }
}

fn parse_preset(s: &str) -> Preset {
    match s.to_lowercase().as_str() {
        "solo" => Preset::Solo,
        "custom" => Preset::Custom,
        "langgraph" => Preset::LangGraph,
        "crewai" | "crew_ai" => Preset::CrewAI,
        "autogen" => Preset::AutoGen,
        "langchain" => Preset::LangChain,
        "openai-swarm" | "openaiswarm" => Preset::OpenAISwarm,
        "semantic-kernel" | "semantickernel" => Preset::SemanticKernel,
        "haystack" => Preset::Haystack,
        "dspy" => Preset::DSPy,
        "llamaindex" | "llama-index" => Preset::LlamaIndex,
        "taskweaver" => Preset::TaskWeaver,
        "agno" => Preset::Agno,
        "atomic-agents" | "atomicagents" => Preset::AtomicAgents,
        "bee-agent" | "beeagent" => Preset::BeeAgent,
        "pydantic-ai" | "pydanticai" => Preset::PydanticAI,
        "claude-code" | "claudecode" => Preset::ClaudeCode,
        "aider" => Preset::Aider,
        "cline" => Preset::Cline,
        "continue" => Preset::Continue,
        "vercel-ai" | "vercelai" => Preset::VercelAI,
        "copilot" => Preset::Copilot,
        "cursor" => Preset::Cursor,
        "windsurf" => Preset::Windsurf,
        "devin" => Preset::Devin,
        "amazon-q" | "amazonq" => Preset::AmazonQ,
        "replit-agent" | "replitagent" => Preset::ReplitAgent,
        "pearai" | "pear-ai" => Preset::PearAI,
        "bolt-new" | "boltnew" => Preset::BoltNew,
        "lovable" => Preset::Lovable,
        "v0" => Preset::V0,
        _ => Preset::Solo,
    }
}
