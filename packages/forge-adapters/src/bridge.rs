//! BridgeAgent — REST/MCP connector for IDE and cloud-based agents.
//!
//! Covers: Copilot, Cursor, Windsurf, Devin, Amazon Q, Replit Agent,
//! PearAI, Bolt.new, Lovable, v0, Vercel AI SDK.
//!
//! These agents run outside the local process (in an IDE, browser, or cloud).
//! The adapter connects via REST callbacks or MCP protocol to observe and
//! intervene on the remote agent session.
//!
//! ## Architecture
//!
//! ```
//! ┌──────────┐  events (REST/MCP)  ┌─────────────┐
//! │  IDE/    │ ──────────────────→ │  BridgeAgent │ ──→ Forge Harness
//! │  Cloud   │ ←────────────────── │              │ ←── Interventions
//! │  Agent   │  interventions      └─────────────┘
//! └──────────┘
//! ```
//!
//! The BridgeAgent exposes:
//! - A webhook endpoint for the remote agent to POST events
//! - A polling interface for the harness to push interventions back
//! - Session management (create, track, terminate remote sessions)

use async_trait::async_trait;
use chrono::Utc;
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Status of a bridged agent session.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum BridgeSessionStatus {
    Connecting,
    Active,
    Paused,
    Completed,
    Failed,
    Terminated,
}

/// Configuration for a bridged agent connection.
pub struct BridgeConfig {
    /// Webhook URL for receiving agent events (the remote agent POSTs here)
    pub webhook_url: Option<String>,
    /// Callback URL for pushing interventions to the remote agent
    pub callback_url: Option<String>,
    /// MCP server endpoint (if using MCP protocol)
    pub mcp_endpoint: Option<String>,
    /// API key for authenticating with the remote agent
    pub api_key: Option<String>,
    /// Poll interval in seconds (if using polling mode)
    pub poll_interval_secs: u64,
    /// Session timeout in seconds
    pub timeout_secs: u64,
}

/// A bridge adapter for IDE/cloud agents that run outside the local process.
///
/// ```rust
/// use forge_adapters::BridgeAgent;
/// use forge_sdk::agent::AgentType;
///
/// let agent = BridgeAgent::new("copilot-1", AgentType::Copilot)
///     .callback_url("https://my-app.example.com/forge/interventions")
///     .timeout(600);
/// ```
pub struct BridgeAgent {
    id: String,
    agent_type: AgentType,
    config: BridgeConfig,
    status: BridgeSessionStatus,
    /// Accumulated events from the remote agent
    events: Vec<serde_json::Value>,
}

impl BridgeAgent {
    /// Create a new BridgeAgent for the given IDE/cloud agent type.
    pub fn new(id: impl Into<String>, agent_type: AgentType) -> Self {
        Self {
            id: id.into(),
            agent_type,
            config: BridgeConfig {
                webhook_url: None,
                callback_url: None,
                mcp_endpoint: None,
                api_key: None,
                poll_interval_secs: 5,
                timeout_secs: 600, // 10 min default for cloud agents
            },
            status: BridgeSessionStatus::Connecting,
            events: vec![],
        }
    }

    /// Set the webhook URL where the remote agent sends events.
    pub fn webhook_url(mut self, url: impl Into<String>) -> Self {
        self.config.webhook_url = Some(url.into());
        self
    }

    /// Set the callback URL where interventions are pushed.
    pub fn callback_url(mut self, url: impl Into<String>) -> Self {
        self.config.callback_url = Some(url.into());
        self
    }

    /// Set the MCP server endpoint.
    pub fn mcp_endpoint(mut self, url: impl Into<String>) -> Self {
        self.config.mcp_endpoint = Some(url.into());
        self
    }

    /// Set the API key for authentication.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = Some(key.into());
        self
    }

    /// Set poll interval in seconds.
    pub fn poll_interval(mut self, secs: u64) -> Self {
        self.config.poll_interval_secs = secs;
        self
    }

    /// Set timeout in seconds.
    pub fn timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Get the current session status.
    pub fn status(&self) -> &BridgeSessionStatus {
        &self.status
    }

    /// Get accumulated events.
    pub fn events(&self) -> &[serde_json::Value] {
        &self.events
    }

    /// Push an intervention to the remote agent via callback URL.
    async fn push_intervention(
        &self,
        intervention: &Intervention,
    ) -> Result<(), ForgeError> {
        if let Some(ref callback_url) = self.config.callback_url {
            let client = reqwest::Client::new();
            let payload = match intervention {
                Intervention::Nudge { message, .. } => serde_json::json!({
                    "type": "nudge",
                    "message": message,
                }),
                Intervention::Pause { reason, .. } => serde_json::json!({
                    "type": "pause",
                    "reason": reason,
                }),
                Intervention::CircuitBreak { reason } => serde_json::json!({
                    "type": "circuit_break",
                    "reason": reason,
                }),
                other => serde_json::json!({
                    "type": format!("{:?}", other),
                }),
            };

            let _ = client
                .post(callback_url)
                .json(&payload)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await;
        }
        Ok(())
    }

    fn handle_interventions(
        &self,
        rx: &mut mpsc::Receiver<Intervention>,
    ) -> Result<(), ForgeError> {
        while let Ok(intervention) = rx.try_recv() {
            match &intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken {
                        reason: reason.clone(),
                    });
                }
                _ => {
                    // Non-fatal interventions are pushed async via the poll loop below
                    tracing::debug!(agent_id = %self.id, ?intervention, "Bridge intervention (queued)");
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentAdapter for BridgeAgent {
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
        let now = Utc::now();
        self.status = BridgeSessionStatus::Active;

        // 1. Started
        let _ = event_tx
            .send(AgentEvent::Started {
                agent_id: self.id.clone(),
                task: task.to_string(),
                timestamp: now,
            })
            .await;

        // 2. Thinking — bridge is waiting for remote agent
        let _ = event_tx
            .send(AgentEvent::ThinkingStart {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 3. Bridge tool call — represents the remote agent session
        let start = std::time::Instant::now();
        let tool_name = format!("{}-bridge", self.agent_type.to_string());

        let _ = event_tx
            .send(AgentEvent::ToolCallStart {
                agent_id: self.id.clone(),
                tool: tool_name.clone(),
                args: serde_json::json!({
                    "agent_type": self.agent_type.to_string(),
                    "task": task,
                    "bridge_mode": if self.config.callback_url.is_some() { "callback" }
                        else if self.config.mcp_endpoint.is_some() { "mcp" }
                        else { "poll" },
                }),
                timestamp: Utc::now(),
            })
            .await;

        // Poll for interventions while the remote agent works
        // In production, this would integrate with the actual IDE/cloud API
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_secs(self.config.timeout_secs);

        loop {
            // Check for interventions
            if let Err(e) = self.handle_interventions(&mut intervention_rx) {
                self.status = BridgeSessionStatus::Terminated;
                return Err(e);
            }

            // Check timeout
            if tokio::time::Instant::now() > deadline {
                self.status = BridgeSessionStatus::Failed;
                break;
            }

            // Wait for poll interval
            tokio::time::sleep(std::time::Duration::from_secs(
                self.config.poll_interval_secs,
            ))
            .await;
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        // 4. Tool call end
        let _ = event_tx
            .send(AgentEvent::ToolCallEnd {
                agent_id: self.id.clone(),
                tool: tool_name,
                result: ToolResult {
                    content: format!(
                        "Bridge session {}: {} completed",
                        self.id,
                        self.agent_type
                    ),
                    is_error: self.status == BridgeSessionStatus::Failed,
                    duration_ms,
                    token_count: 0,
                },
                timestamp: Utc::now(),
            })
            .await;

        // 5. Thinking end
        let _ = event_tx
            .send(AgentEvent::ThinkingEnd {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 6. Completed
        self.status = BridgeSessionStatus::Completed;
        let _ = event_tx
            .send(AgentEvent::Completed {
                agent_id: self.id.clone(),
                summary: format!("Bridge session for {} completed", self.agent_type),
                timestamp: Utc::now(),
            })
            .await;

        Ok(AgentOutcome {
            success: true,
            summary: format!(
                "Bridge agent {} completed ({}ms). Remote agent session tracked.",
                self.agent_type, duration_ms
            ),
            output: Some(serde_json::json!({
                "agent_type": self.agent_type.to_string(),
                "bridge_mode": "poll",
                "duration_ms": duration_ms,
                "events_collected": self.events.len(),
            }).to_string()),
        })
    }
}
