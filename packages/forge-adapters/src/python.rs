//! PythonAgent — Subprocess adapter for Python-based AI frameworks.
//!
//! Covers: LangGraph, CrewAI, AutoGen, LangChain, OpenAI Swarm,
//! Semantic Kernel (Python), Haystack, DSPy, LlamaIndex, TaskWeaver,
//! Agno, Atomic Agents, and Bee Agent Framework.
//!
//! Each agent type gets a template Python script that:
//! 1. Imports the framework
//! 2. Creates an agent instance
//! 3. Runs the task
//! 4. Outputs JSON result to stdout
//!
//! The adapter spawns `python -c "<script>"` with the task injected.

use async_trait::async_trait;
use chrono::Utc;
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::mpsc;

/// A real AgentAdapter that wraps a Python-based AI framework.
///
/// ```rust
/// use forge_adapters::PythonAgent;
/// use forge_sdk::agent::AgentType;
///
/// let agent = PythonAgent::new("langgraph-1", AgentType::LangGraph)
///     .script(r#"
/// import json, sys
/// # Your LangGraph agent code here
/// graph = create_graph()
/// result = graph.invoke({"task": sys.argv[1]})
/// print(json.dumps({"success": True, "output": str(result)}).to_string()
/// "#);
/// ```
pub struct PythonAgent {
    id: String,
    agent_type: AgentType,
    script: Option<String>,
    python_path: String,
    work_dir: String,
    timeout_secs: Option<u64>,
    env: Vec<(String, String)>,
}

impl PythonAgent {
    /// Create a new PythonAgent with a sensible template for the given agent type.
    pub fn new(id: impl Into<String>, agent_type: AgentType) -> Self {
        Self {
            id: id.into(),
            agent_type,
            script: None,
            python_path: "python".into(),
            work_dir: ".".into(),
            timeout_secs: Some(300),
            env: vec![],
        }
    }

    /// Provide a custom Python script that receives the task as sys.argv[1].
    /// Must print JSON to stdout: {"success": true/false, "output": "..."}
    pub fn script(mut self, script: impl Into<String>) -> Self {
        self.script = Some(script.into());
        self
    }

    /// Set the Python executable path.
    pub fn python_path(mut self, path: impl Into<String>) -> Self {
        self.python_path = path.into();
        self
    }

    /// Set working directory.
    pub fn work_dir(mut self, dir: impl Into<String>) -> Self {
        self.work_dir = dir.into();
        self
    }

    /// Set timeout in seconds.
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = Some(secs);
        self
    }

    /// No timeout.
    pub fn no_timeout(mut self) -> Self {
        self.timeout_secs = None;
        self
    }

    /// Add an environment variable.
    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.push((key.into(), val.into()));
        self
    }

    /// Get the default script template for this agent type.
    fn default_script(&self) -> String {
        let import = match self.agent_type {
            AgentType::LangGraph => "from langgraph.graph import StateGraph",
            AgentType::CrewAI => "from crewai import Agent, Task, Crew",
            AgentType::AutoGen => "import autogen",
            AgentType::LangChain => "from langchain.chains import LLMChain",
            AgentType::OpenAISwarm => "from swarm import Swarm, Agent",
            AgentType::SemanticKernel => "import semantic_kernel as sk",
            AgentType::Haystack => "from haystack import Pipeline",
            AgentType::DSPy => "import dspy",
            AgentType::LlamaIndex => "from llama_index.core import VectorStoreIndex",
            AgentType::TaskWeaver => "from taskweaver.app.app import TaskWeaverApp",
            AgentType::Agno => "from agno import Agent",
            AgentType::AtomicAgents => "from atomic_agents import Agent",
            AgentType::BeeAgent => "from bee_agent import BeeAgent",
            _ => "# No specific framework import needed",
        };

        format!(
            r#"
import sys, json, traceback
try:
    {import}
    task = sys.argv[1] if len(sys.argv) > 1 else "No task provided"

    # TODO: User must provide their agent implementation.
    # This template is a starting point — customize it for your use case.
    #
    # Example for LangGraph:
    #   graph = build_my_graph()
    #   result = graph.invoke({{"task": task}})
    #   print(json.dumps({{"success": True, "output": str(result)}}).to_string()
    #
    # Example for CrewAI:
    #   agent = Agent(role="assistant", goal=task, backstory="...", allow_delegation=False)
    #   task_obj = Task(description=task, expected_output="...", agent=agent)
    #   crew = Crew(agents=[agent], tasks=[task_obj])
    #   result = crew.kickoff()
    #   print(json.dumps({{"success": True, "output": str(result)}}).to_string()

    print(json.dumps({{
        "success": False,
        "output": "Agent script not configured. Set up your {{}} agent logic in the script.",
        "framework": "{}"
    }}).to_string()
except Exception as e:
    print(json.dumps({{
        "success": False,
        "output": str(e),
        "traceback": traceback.format_exc()
    }}).to_string()
"#,
            self.agent_type,
        )
    }

    fn handle_interventions(
        &self,
        rx: &mut mpsc::Receiver<Intervention>,
    ) -> Result<(), ForgeError> {
        while let Ok(intervention) = rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                _ => {
                    tracing::debug!(agent_id = %self.id, ?intervention, "Intervention");
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentAdapter for PythonAgent {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        mut intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError> {
        let script = self
            .script
            .clone()
            .unwrap_or_else(|| self.default_script());

        let now = Utc::now();

        // 1. Started
        let _ = event_tx
            .send(AgentEvent::Started {
                agent_id: self.id.clone(),
                task: task.to_string(),
                timestamp: now,
            })
            .await;

        self.handle_interventions(&mut intervention_rx)?;

        // 2. Thinking
        let _ = event_tx
            .send(AgentEvent::ThinkingStart {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 3. Spawn python
        let mut cmd = Command::new(&self.python_path);
        cmd.arg("-c").arg(&script);
        cmd.arg(task);
        cmd.current_dir(&self.work_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);
        for (k, v) in &self.env {
            cmd.env(k, v);
        }

        let start = std::time::Instant::now();
        let output = if let Some(timeout_secs) = self.timeout_secs {
            let child = cmd.spawn().map_err(|e| ForgeError::ToolExecution(
                format!("Failed to spawn Python: {}", e)))?;
            tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs),
                child.wait_with_output(),
            )
            .await
            .map_err(|_| ForgeError::ToolExecution(
                format!("Python agent timed out after {}s", timeout_secs)))?
            .map_err(|e| ForgeError::ToolExecution(
                format!("Python process error: {}", e)))?
        } else {
            cmd.output().await.map_err(|e| ForgeError::ToolExecution(
                format!("Failed to run Python: {}", e)))?
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let is_error = !output.status.success();

        // Try to parse JSON output
        let parsed: serde_json::Value =
            serde_json::from_str(&stdout).unwrap_or(serde_json::json!({
                "success": !is_error,
                "output": stdout,
                "stderr": stderr,
            }));

        let result_success = parsed["success"].as_bool().unwrap_or(!is_error);
        let result_output = parsed["output"].as_str().unwrap_or(&stdout).to_string();

        // 4. Tool call
        let tool_name = self.agent_type.to_string();
        let _ = event_tx
            .send(AgentEvent::ToolCallStart {
                agent_id: self.id.clone(),
                tool: tool_name.clone(),
                args: serde_json::json!({"task": task}),
                timestamp: Utc::now(),
            })
            .await;

        let _ = event_tx
            .send(AgentEvent::ToolCallEnd {
                agent_id: self.id.clone(),
                tool: tool_name,
                result: ToolResult {
                    content: result_output.clone(),
                    is_error: !result_success,
                    duration_ms,
                    token_count: (stdout.len() / 4) as u64,
                },
                timestamp: Utc::now(),
            })
            .await;

        self.handle_interventions(&mut intervention_rx)?;

        // 5. Thinking end
        let _ = event_tx
            .send(AgentEvent::ThinkingEnd {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 6. Completed
        let _ = event_tx
            .send(AgentEvent::Completed {
                agent_id: self.id.clone(),
                summary: result_output.chars().take(300).collect(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(AgentOutcome {
            success: result_success,
            summary: format!(
                "{} agent completed in {}ms",
                self.agent_type, duration_ms
            ),
            output: Some(parsed.to_string()),
        })
    }
}
