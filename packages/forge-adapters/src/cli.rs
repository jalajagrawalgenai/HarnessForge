//! CliAgent — Generic CLI subprocess adapter.
//!
//! Wraps any CLI-based AI agent (Claude Code, Aider, Cline, Continue)
//! by spawning it as a subprocess, capturing stdout/stderr as tool output,
//! and checking for harness interventions between executions.
//!
//! ## Supported Agents
//!
//! | Agent | CLI Command | Default Flags |
//! |---|---|---|
//! | Claude Code | `claude` | `-p` (print, non-interactive) |
//! | Aider | `aider` | `--message` |
//! | Cline | `cline` | `--cli` |
//! | Continue | `continue` | `--chat` |

use async_trait::async_trait;
use chrono::Utc;
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::mpsc;

/// Configuration for a CLI-based agent.
pub struct CliAgentConfig {
    /// The executable name (e.g. "claude", "aider")
    pub command: String,
    /// Default arguments before the task (e.g. ["-p"] for Claude Code print mode)
    pub default_args: Vec<String>,
    /// Working directory for the subprocess
    pub work_dir: String,
    /// Environment variables to pass
    pub env: Vec<(String, String)>,
    /// Timeout in seconds (None = no timeout)
    pub timeout_secs: Option<u64>,
}

/// A real AgentAdapter that wraps any CLI-based AI agent.
///
/// ```rust
/// use forge_adapters::CliAgent;
/// use forge_sdk::agent::AgentType;
///
/// let agent = CliAgent::new("claude-1", AgentType::ClaudeCode)
///     .command("claude")
///     .args(vec!["-p", "--output-format", "stream-json"]);
/// ```
pub struct CliAgent {
    id: String,
    agent_type: AgentType,
    config: CliAgentConfig,
}

impl CliAgent {
    /// Create a new CliAgent with sensible defaults for the given agent type.
    pub fn new(id: impl Into<String>, agent_type: AgentType) -> Self {
        let (cmd, args) = match agent_type {
            AgentType::ClaudeCode => ("claude", vec!["-p".into()]),
            AgentType::Aider => ("aider", vec!["--message".into(), "--no-git".into()]),
            AgentType::Cline => ("cline", vec!["--cli".into()]),
            AgentType::Continue => ("continue", vec!["--chat".into()]),
            _ => ("claude", vec!["-p".into()]), // default fallback
        };

        Self {
            id: id.into(),
            agent_type,
            config: CliAgentConfig {
                command: cmd.into(),
                default_args: args,
                work_dir: ".".into(),
                env: vec![],
                timeout_secs: Some(300), // 5 min default
            },
        }
    }

    /// Override the CLI command.
    pub fn command(mut self, cmd: impl Into<String>) -> Self {
        self.config.command = cmd.into();
        self
    }

    /// Override default arguments.
    pub fn args(mut self, args: Vec<impl Into<String>>) -> Self {
        self.config.default_args = args.into_iter().map(|a| a.into()).collect();
        self
    }

    /// Set working directory.
    pub fn work_dir(mut self, dir: impl Into<String>) -> Self {
        self.config.work_dir = dir.into();
        self
    }

    /// Add an environment variable.
    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.config.env.push((key.into(), val.into()));
        self
    }

    /// Set timeout in seconds.
    pub fn timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = Some(secs);
        self
    }

    /// No timeout.
    pub fn no_timeout(mut self) -> Self {
        self.config.timeout_secs = None;
        self
    }

    /// Build the full command with task substituted in.
    fn build_command(&self, task: &str) -> Command {
        let mut cmd = Command::new(&self.config.command);
        cmd.args(&self.config.default_args);
        cmd.arg(task);
        cmd.current_dir(&self.config.work_dir);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.kill_on_drop(true);
        for (k, v) in &self.config.env {
            cmd.env(k, v);
        }
        cmd
    }

    /// Process harness interventions.
    fn handle_interventions(
        &self,
        intervention_rx: &mut mpsc::Receiver<Intervention>,
    ) -> Result<(), ForgeError> {
        while let Ok(intervention) = intervention_rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                Intervention::Pause { reason, .. } => {
                    tracing::warn!(
                        agent_id = %self.id,
                        reason = %reason,
                        "Agent paused by harness — awaiting human"
                    );
                }
                Intervention::Nudge { message, .. } => {
                    tracing::info!(
                        agent_id = %self.id,
                        message = %message,
                        "Harness nudge received"
                    );
                }
                Intervention::Compact { target_ratio, .. } => {
                    tracing::info!(
                        agent_id = %self.id,
                        target = %target_ratio,
                        "Harness compaction requested"
                    );
                }
                _ => {
                    tracing::debug!(
                        agent_id = %self.id,
                        intervention = ?intervention,
                        "Harness intervention received"
                    );
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentAdapter for CliAgent {
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

        // 1. Signal: agent is starting
        let _ = event_tx
            .send(AgentEvent::Started {
                agent_id: self.id.clone(),
                task: task.to_string(),
                timestamp: now,
            })
            .await;

        // 2. Check for interventions before starting
        self.handle_interventions(&mut intervention_rx)?;

        // 3. Signal: agent is thinking
        let _ = event_tx
            .send(AgentEvent::ThinkingStart {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 4. Spawn the CLI tool as a subprocess
        let start = std::time::Instant::now();
        let mut cmd = self.build_command(task);

        let output = if let Some(timeout_secs) = self.config.timeout_secs {
            let child = cmd.spawn().map_err(|e| ForgeError::ToolExecution(
                format!("Failed to spawn {}: {}", self.config.command, e)))?;

            let output =
                tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), child.wait_with_output())
                    .await
                    .map_err(|_| ForgeError::ToolExecution(
                        format!(
                            "{} timed out after {} seconds",
                            self.config.command, timeout_secs
                        )))?
                    .map_err(|e| ForgeError::ToolExecution(
                        format!("{} process error: {}", self.config.command, e)))?;
            output
        } else {
            cmd.output().await.map_err(|e| ForgeError::ToolExecution(
                format!("Failed to run {}: {}", self.config.command, e)))?
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let is_error = !output.status.success();

        // 5. Report the tool call result
        let _ = event_tx
            .send(AgentEvent::ToolCallStart {
                agent_id: self.id.clone(),
                tool: self.config.command.clone(),
                args: serde_json::json!({"task": task}),
                timestamp: Utc::now(),
            })
            .await;

        let _ = event_tx
            .send(AgentEvent::ToolCallEnd {
                agent_id: self.id.clone(),
                tool: self.config.command.clone(),
                result: ToolResult {
                    content: if is_error && !stderr.is_empty() {
                        format!("STDOUT:\n{}\n\nSTDERR:\n{}", stdout, stderr)
                    } else {
                        stdout.clone()
                    },
                    is_error,
                    duration_ms,
                    token_count: (stdout.len() / 4) as u64, // rough estimate: 4 chars ≈ 1 token
                },
                timestamp: Utc::now(),
            })
            .await;

        // 6. Check for interventions after execution
        self.handle_interventions(&mut intervention_rx)?;

        // 7. Signal thinking end
        let _ = event_tx
            .send(AgentEvent::ThinkingEnd {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 8. Signal completion
        let _ = event_tx
            .send(AgentEvent::Completed {
                agent_id: self.id.clone(),
                summary: stdout.chars().take(300).collect(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(AgentOutcome {
            success: !is_error,
            summary: format!(
                "{} completed in {}ms ({} chars output)",
                self.config.command,
                duration_ms,
                stdout.len()
            ),
            output: Some(serde_json::json!({
                "command": self.config.command,
                "duration_ms": duration_ms,
                "exit_code": output.status.code(),
                "stdout_len": stdout.len(),
                "stderr_len": stderr.len(),
            }).to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_agent_defaults_per_type() {
        let claude = CliAgent::new("test", AgentType::ClaudeCode);
        assert_eq!(claude.config.command, "claude");
        assert!(claude.config.default_args.contains(&"-p".to_string()));

        let aider = CliAgent::new("test", AgentType::Aider);
        assert_eq!(aider.config.command, "aider");
        assert!(aider.config.default_args.contains(&"--message".to_string()));

        let cline = CliAgent::new("test", AgentType::Cline);
        assert_eq!(cline.config.command, "cline");

        let cont = CliAgent::new("test", AgentType::Continue);
        assert_eq!(cont.config.command, "continue");
    }

    #[test]
    fn test_cli_agent_builder() {
        let agent = CliAgent::new("custom-1", AgentType::Custom)
            .command("my-cli")
            .args(vec!["--run", "--json"])
            .work_dir("/tmp/project")
            .env("API_KEY", "test-key")
            .timeout(60);

        assert_eq!(agent.config.command, "my-cli");
        assert_eq!(agent.config.default_args, vec!["--run", "--json"]);
        assert_eq!(agent.config.work_dir, "/tmp/project");
        assert_eq!(agent.config.env, vec![("API_KEY".into(), "test-key".into())]);
        assert_eq!(agent.config.timeout_secs, Some(60));
    }
}
