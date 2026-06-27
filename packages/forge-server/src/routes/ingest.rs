//! Event ingestion route — universal entry point for ALL agentic systems.
//!
//! POST /api/v1/ingest/event
//!
//! Accepts events from any agentic framework (Claude Code hooks, LangGraph
//! callbacks, CrewAI middleware, AutoGen observers, etc.). Auto-creates
//! sessions on first contact. Feeds events into the harness pipeline.
//! Returns pending interventions when applicable.
//!
//! POST /api/v1/ingest/transcript
//!
//! Accepts transcript summary data after session completion (token usage,
//! model breakdown, cache metrics) for analytics and audit.

use crate::session::store::{SessionState, SessionStatus};
use crate::AppState;
use axum::extract::State;
use axum::Json;
use chrono::Utc;
use forge_sdk::events::AgentEvent;
use serde_json::{json, Value};
use std::sync::Arc;

/// POST /api/v1/ingest/event
///
/// Universal event ingestion. Accepts events from any agentic system.
/// Auto-creates sessions when a lifecycle-start event arrives.
/// Feeds events into the harness pipeline for observation/detection/intervention.
///
/// Expected envelope:
/// ```json
/// {
///   "agentClass": "claude-code | langgraph | crewai | autogen | ...",
///   "sessionId": "...",
///   "agentId": "...",
///   "hookName": "SessionStart | UserPromptSubmit | PreToolUse | PostToolUse | ...",
///   "toolName": "...",
///   "payload": { ... },
///   "cwd": "...",
///   "timestamp": 1234567890,
///   "_meta": { "agent": {...}, "session": {...}, "project": {...} },
///   "flags": { "startsSession": true, "stopsSession": false, ... }
/// }
/// ```
pub async fn ingest_event(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let hook_name = body
        .get("hookName")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let session_id = body
        .get("sessionId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let agent_id = body
        .get("agentId")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    let agent_class = body
        .get("agentClass")
        .and_then(|v| v.as_str())
        .unwrap_or("default");
    let tool_name = body.get("toolName").and_then(|v| v.as_str()).unwrap_or("");
    let timestamp = body.get("timestamp").and_then(|v| v.as_u64()).unwrap_or(0);
    let flags = body.get("flags").cloned().unwrap_or(json!({}));
    let payload = body.get("payload").cloned().unwrap_or(json!({}));

    let starts_session = flags
        .get("startsSession")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let stops_session = flags
        .get("stopsSession")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // ── Auto-create session on first contact ──
    {
        let sessions = state.store.read().await;
        if !sessions.contains_key(session_id) {
            drop(sessions);
            let task = payload
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("(live agent session)")
                .to_string();

            // Map agent_class to our agent_type
            let agent_type = map_agent_class(agent_class);
            let preset = agent_type.clone(); // default preset matches agent type

            let session = SessionState::new(session_id.to_string(), task, agent_type, preset);

            let mut sessions = state.store.write().await;
            sessions.insert(session_id.to_string(), session);
            tracing::info!(
                session_id = %session_id,
                agent_class = %agent_class,
                "Auto-created session from ingest event"
            );
        }
    }

    // ── Convert hook event to AgentEvent and broadcast ──
    let agent_event = hook_to_agent_event(hook_name, agent_id, tool_name, &payload, timestamp);

    if let Some(event) = agent_event {
        // Broadcast to WebSocket/SSE consumers
        if let Some(broadcaster) = {
            let sessions = state.store.read().await;
            sessions
                .get(session_id)
                .map(|s| s.event_broadcaster.clone())
        } {
            let _ = broadcaster.send(event.clone());
        }

        // Store in ring buffer
        {
            let mut sessions = state.store.write().await;
            if let Some(s) = sessions.get_mut(session_id) {
                s.events.push(event.clone());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                }
            }
        }
    }

    // ── Update session status based on flags ──
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            if starts_session && s.status == SessionStatus::Pending {
                s.status = SessionStatus::Running;
            }
            if stops_session {
                s.status = SessionStatus::Completed;
                s.completed_at = Some(Utc::now());
            }
            // Update task if payload has a prompt
            if let Some(prompt) = payload.get("prompt").and_then(|v| v.as_str()) {
                if s.task == "(live agent session)" && !prompt.is_empty() {
                    s.task = prompt.to_string();
                }
            }
            // Track tool usage counts
            if hook_name == "PostToolUse" && !tool_name.is_empty() {
                // Increment observation count via detection counter
                // (actual detection runs in pipeline cycle)
            }
        }
    }

    // ── Return response ──
    // Check if there are pending interventions for this session
    let interventions: Vec<Value> = vec![];
    // TODO: query pipeline for pending interventions

    Json(json!({
        "status": "ok",
        "sessionId": session_id,
        "hookName": hook_name,
        "ingested": true,
        "interventions": interventions,
    }))
}

/// POST /api/v1/ingest/transcript
///
/// Accepts transcript summary after session completion.
/// Stores token usage, model breakdown, cache metrics.
pub async fn ingest_transcript(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let session_id = body.get("sessionId").and_then(|v| v.as_str()).unwrap_or("");

    tracing::info!(
        session_id = %session_id,
        "Received transcript data"
    );

    // Store transcript metrics in session
    {
        let mut sessions = state.store.write().await;
        if let Some(s) = sessions.get_mut(session_id) {
            // Record token usage event if we have data
            let input = body
                .get("totalInputTokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let output = body
                .get("totalOutputTokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_read = body
                .get("totalCacheRead")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let cache_write = body
                .get("totalCacheWrite")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let models: Vec<String> = body
                .get("models")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            if input > 0 || output > 0 {
                let token_event = AgentEvent::TokenUsage {
                    agent_id: s.id.clone(),
                    input,
                    output,
                    cache_read,
                    cache_write,
                    model: models.first().cloned().unwrap_or_else(|| "unknown".into()),
                    timestamp: Utc::now(),
                };
                s.events.push(token_event.clone());
                if s.events.len() > 1000 {
                    s.events.remove(0);
                }
                let _ = s.event_broadcaster.send(token_event);
            }
        }
    }

    Json(json!({
        "status": "ok",
        "sessionId": session_id,
        "synced": true
    }))
}

/// GET /api/v1/ingest/status
///
/// Returns ingestion stats: active sessions, event counts, etc.
pub async fn ingest_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.store.read().await;
    let total = sessions.len();
    let running = sessions
        .values()
        .filter(|s| s.status == SessionStatus::Running)
        .count();
    let total_events: usize = sessions.values().map(|s| s.events.len()).sum();
    let auto_created = sessions
        .values()
        .filter(|s| s.task == "(live agent session)")
        .count();

    Json(json!({
        "status": "ok",
        "activeSessions": running,
        "totalSessions": total,
        "totalEventsInRing": total_events,
        "autoCreatedSessions": auto_created,
        "message": if running > 0 {
            format!("{} agent(s) being monitored", running)
        } else {
            "Waiting for agent activity...".into()
        }
    }))
}

// ── Helpers ──

/// Map an agent class string (from hook envelope) to our AgentType string.
fn map_agent_class(ac: &str) -> String {
    match ac.to_lowercase().as_str() {
        "claude-code" | "claudecode" | "claude" => "claude-code",
        "langgraph" => "langgraph",
        "crewai" | "crew" => "crewai",
        "autogen" => "autogen",
        "langchain" => "langchain",
        "openai-swarm" | "swarm" => "openai-swarm",
        "semantic-kernel" | "sk" => "semantic-kernel",
        "haystack" => "haystack",
        "dspy" => "dspy",
        "llamaindex" | "llama-index" => "llamaindex",
        "taskweaver" => "taskweaver",
        "agno" => "agno",
        "atomic-agents" | "atomic" => "atomic-agents",
        "bee-agent" | "bee" => "bee-agent",
        "pydantic-ai" | "pydantic" => "pydantic-ai",
        "aider" => "aider",
        "cline" => "cline",
        "continue" => "continue",
        "vercel-ai" | "vercel" => "vercel-ai",
        "copilot" => "copilot",
        "cursor" => "cursor",
        "windsurf" => "windsurf",
        "devin" => "devin",
        "amazon-q" | "q" => "amazon-q",
        "replit-agent" | "replit" => "replit-agent",
        "pearai" | "pear" => "pearai",
        "bolt-new" | "bolt" => "bolt-new",
        "lovable" => "lovable",
        "v0" => "v0",
        "solo" | "custom" | "default" => "solo",
        _ => "solo",
    }
    .to_string()
}

/// Convert a Claude Code / agentic hook event into a harness AgentEvent.
///
/// This is the universal event mapper. It handles hook events from Claude Code,
/// LangGraph callbacks, CrewAI step events, AutoGen messages, etc.
fn hook_to_agent_event(
    hook_name: &str,
    agent_id: &str,
    tool_name: &str,
    payload: &Value,
    timestamp_ms: u64,
) -> Option<AgentEvent> {
    let ts = if timestamp_ms > 0 {
        // Convert epoch millis to DateTime<Utc>
        let secs = (timestamp_ms / 1000) as i64;
        let nsecs = ((timestamp_ms % 1000) * 1_000_000) as u32;
        chrono::DateTime::from_timestamp(secs, nsecs).unwrap_or_else(Utc::now)
    } else {
        Utc::now()
    };

    match hook_name {
        // ── Lifecycle ──
        "SessionStart" | "Setup" => {
            let task = payload
                .get("prompt")
                .and_then(|v| v.as_str())
                .unwrap_or("(session started)");
            Some(AgentEvent::Started {
                agent_id: agent_id.to_string(),
                task: task.to_string(),
                timestamp: ts,
            })
        }
        "SessionEnd" | "Stop" => Some(AgentEvent::Completed {
            agent_id: agent_id.to_string(),
            summary: "Session completed".into(),
            timestamp: ts,
        }),
        "StopFailure" => {
            let error = payload
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            Some(AgentEvent::Failed {
                agent_id: agent_id.to_string(),
                error: error.to_string(),
                timestamp: ts,
            })
        }

        // ── User input ──
        "UserPromptSubmit" => {
            let text = payload
                .get("prompt")
                .or(payload.get("text"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            Some(AgentEvent::MessageSent {
                from: "user".into(),
                to: vec![agent_id.to_string()],
                content: forge_sdk::events::MessageContent::Text(text.to_string()),
                timestamp: ts,
            })
        }

        // ── Tool calls ──
        "PreToolUse" => {
            let args = payload.get("tool_input").cloned().unwrap_or(json!({}));
            Some(AgentEvent::ToolCallStart {
                agent_id: agent_id.to_string(),
                tool: tool_name.to_string(),
                args,
                timestamp: ts,
            })
        }
        "PostToolUse" => {
            // Extract tool result from response
            let tool_response = payload.get("tool_response");
            let content = tool_response.and_then(|v| v.as_str()).unwrap_or("");
            let is_error = payload
                .get("is_error")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Try to extract usage from tool_response
            let usage = payload.get("tool_use_result").or(tool_response);
            let token_count = usage
                .and_then(|v| v.get("usage"))
                .and_then(|v| v.get("input_tokens"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            Some(AgentEvent::ToolCallEnd {
                agent_id: agent_id.to_string(),
                tool: tool_name.to_string(),
                result: forge_sdk::events::ToolResult {
                    content: content.to_string(),
                    is_error,
                    duration_ms: 0, // extracted from timing data if available
                    token_count,
                },
                timestamp: ts,
            })
        }
        "PostToolUseFailure" => Some(AgentEvent::ToolCallEnd {
            agent_id: agent_id.to_string(),
            tool: tool_name.to_string(),
            result: forge_sdk::events::ToolResult {
                content: payload
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("tool failed")
                    .to_string(),
                is_error: true,
                duration_ms: 0,
                token_count: 0,
            },
            timestamp: ts,
        }),

        // ── Thinking / Reasoning ──
        "PreCompact" => {
            let ratio = payload
                .get("context_ratio")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.85);
            Some(AgentEvent::ContextPressure {
                agent_id: agent_id.to_string(),
                current_ratio: ratio,
                trend: 0.05,
                timestamp: ts,
            })
        }

        // ── Sub-agents ──
        "SubagentStart" => {
            let child_id = payload
                .get("subagent_id")
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id);
            let task = payload
                .get("subagent_name")
                .and_then(|v| v.as_str())
                .unwrap_or("subagent task");
            Some(AgentEvent::Forked {
                parent_id: agent_id.to_string(),
                child_id: child_id.to_string(),
                task: task.to_string(),
                timestamp: ts,
            })
        }
        "SubagentStop" => Some(AgentEvent::Completed {
            agent_id: payload
                .get("subagent_id")
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id)
                .to_string(),
            summary: "Subagent finished".into(),
            timestamp: ts,
        }),

        // ── Configuration / Context ──
        "CwdChanged" | "FileChanged" | "ConfigChange" => Some(AgentEvent::StateTransition {
            agent_id: agent_id.to_string(),
            from: "previous".into(),
            to: hook_name.to_string(),
            condition: payload
                .get("path")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            timestamp: ts,
        }),

        // ── Output (for streaming agents) ──
        "Notification" => {
            let text = payload
                .get("message")
                .or(payload.get("notification"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if !text.is_empty() {
                Some(AgentEvent::OutputDelta {
                    agent_id: agent_id.to_string(),
                    text: text.to_string(),
                    timestamp: ts,
                })
            } else {
                None
            }
        }

        // ── Other events we store but don't map to specific AgentEvent ──
        "UserPromptExpansion"
        | "PostToolBatch"
        | "PermissionRequest"
        | "PermissionDenied"
        | "TeammateIdle"
        | "TaskCreated"
        | "TaskCompleted"
        | "InstructionsLoaded"
        | "PostCompact"
        | "Elicitation"
        | "ElicitationResult"
        | "WorktreeRemove" => {
            // Store as a lightweight output or state transition
            Some(AgentEvent::OutputDelta {
                agent_id: agent_id.to_string(),
                text: format!(
                    "[{}] {}",
                    hook_name,
                    payload
                        .get("summary")
                        .or(payload.get("description"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                ),
                timestamp: ts,
            })
        }

        _ => {
            // Unknown event type — still store for audit
            Some(AgentEvent::OutputDelta {
                agent_id: agent_id.to_string(),
                text: format!("[{}]", hook_name),
                timestamp: ts,
            })
        }
    }
}
