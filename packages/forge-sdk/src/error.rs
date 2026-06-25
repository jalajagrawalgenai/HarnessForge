// forge-sdk/src/error.rs — Error types

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ForgeError {
    #[error("LLM API error: {0}")]
    LlmApi(String),

    #[error("Tool execution error: {0}")]
    ToolExecution(String),

    #[error("Context window exceeded: {used} > {limit}")]
    ContextWindowExceeded { used: usize, limit: usize },

    #[error("Rate limited, retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Checkpoint error: {0}")]
    Checkpoint(String),

    #[error("Audit error: {0}")]
    Audit(String),

    #[error("Intervention failed: {0}")]
    InterventionFailed(String),

    #[error("Circuit broken: {reason}")]
    CircuitBroken { reason: String },
}

pub type ForgeResult<T> = Result<T, ForgeError>;
