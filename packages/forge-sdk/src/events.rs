// forge-sdk/src/events.rs — Core event types flowing through the harness bus

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Every agent event flows through the harness bus.
/// The harness observes all events. It can also INJECT interventions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    // ─── Lifecycle ───
    Started {
        agent_id: String,
        task: String,
        timestamp: DateTime<Utc>,
    },
    Completed {
        agent_id: String,
        summary: String,
        timestamp: DateTime<Utc>,
    },
    Failed {
        agent_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    Forked {
        parent_id: String,
        child_id: String,
        task: String,
        timestamp: DateTime<Utc>,
    },

    // ─── Reasoning ───
    ThinkingStart {
        agent_id: String,
        timestamp: DateTime<Utc>,
    },
    ThinkingDelta {
        agent_id: String,
        text: String,
        timestamp: DateTime<Utc>,
    },
    ThinkingEnd {
        agent_id: String,
        timestamp: DateTime<Utc>,
    },

    // ─── Tool Calls ───
    ToolCallStart {
        agent_id: String,
        tool: String,
        args: serde_json::Value,
        timestamp: DateTime<Utc>,
    },
    ToolCallEnd {
        agent_id: String,
        tool: String,
        result: ToolResult,
        timestamp: DateTime<Utc>,
    },
    ToolCallCached {
        agent_id: String,
        tool: String,
        timestamp: DateTime<Utc>,
    },

    // ─── Communication (multi-agent) ───
    MessageSent {
        from: String,
        to: Vec<String>,
        content: MessageContent,
        timestamp: DateTime<Utc>,
    },
    MessageReceived {
        from: String,
        to: String,
        content: MessageContent,
        timestamp: DateTime<Utc>,
    },

    // ─── Resources ───
    TokenUsage {
        agent_id: String,
        input: u64,
        output: u64,
        cache_read: u64,
        cache_write: u64,
        model: String,
        timestamp: DateTime<Utc>,
    },
    ContextPressure {
        agent_id: String,
        current_ratio: f64,
        trend: f64,
        timestamp: DateTime<Utc>,
    },

    // ─── State (graph-based agents) ───
    StateTransition {
        agent_id: String,
        from: String,
        to: String,
        condition: String,
        timestamp: DateTime<Utc>,
    },

    // ─── Output ───
    OutputDelta {
        agent_id: String,
        text: String,
        timestamp: DateTime<Utc>,
    },
    OutputComplete {
        agent_id: String,
        content: String,
        timestamp: DateTime<Utc>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: String,
    pub is_error: bool,
    pub duration_ms: u64,
    pub token_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageContent {
    Text(String),
    ToolCall { name: String, args: serde_json::Value },
    Task { description: String },
    Structured(serde_json::Value),
}

/// Interventions are injected by the harness into agent streams.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Intervention {
    /// Inject a hint into agent context
    Nudge {
        message: String,
        reason: String,
    },
    /// Insert message as if from user (stronger than nudge)
    Interject {
        message: String,
        reason: String,
    },
    /// Trigger context compaction
    Compact {
        target_ratio: f64,
        layer: CompressionLayer,
    },
    /// Pause agent, wait for human
    Pause {
        reason: String,
        checkpoint_id: Uuid,
    },
    /// Resume after pause
    Resume,
    /// Upgrade model, expand budget, add tools
    Escalate {
        new_model: Option<String>,
        budget_increase: Option<u64>,
        reason: String,
    },
    /// Fork agent into N children
    Fork {
        count: u32,
        subtasks: Vec<String>,
    },
    /// Change agent's next action (graph mode)
    Reroute {
        to_node: String,
        reason: String,
    },
    /// Restore from checkpoint
    Rollback {
        checkpoint_id: Uuid,
        reason: String,
    },
    /// Force agent to try different approach
    Diversify {
        alternative_approach: String,
    },
    /// Remove dangerous tools, restrict context
    Isolate {
        level: IsolationLevel,
        reason: String,
    },
    /// Emergency stop all agents
    CircuitBreak {
        reason: String,
    },
    /// Kill agent, spawn replacement
    Replace {
        context_summary: String,
        new_model: Option<String>,
    },
    /// Switch to cheaper model, remove expensive tools
    Degrade {
        level: DegradeLevel,
    },
    /// Route agent output to sandbox
    Quarantine {
        reason: String,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionLayer {
    Budget,
    Snip,
    Microcompact,
    Autocompact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    ToolRestrict,
    ContextRestrict,
    FullSandbox,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DegradeLevel {
    Mild,
    Moderate,
    Severe,
}

/// Outcome returned when agent completes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutcome {
    pub success: bool,
    pub summary: String,
    pub output: Option<String>,
}
