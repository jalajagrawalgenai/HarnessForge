// forge-sdk/src/prelude.rs — Convenient re-exports

pub use crate::agent::{AgentAdapter, AgentType, BridgeMethod};
pub use crate::error::{ForgeError, ForgeResult};
pub use crate::events::{
    AgentEvent, AgentOutcome, CompressionLayer, DegradeLevel, Intervention, IsolationLevel,
    MessageContent, ToolResult,
};
pub use crate::harness::{
    Harness, HarnessBuilder, HarnessConfig, HarnessRunResult, HarnessRuntime,
};
pub use crate::traits::detector::Detector;
pub use crate::traits::observer::Observer;
pub use crate::traits::store::AuditStore;
pub use crate::traits::strategy::Strategy;
pub use crate::types::audit::{
    AuditEvent, AuditPhase, AuditReport, Checkpoint, CheckpointSummary, DetectionSummary,
    InterventionSummary, ObservationSummary,
};
pub use crate::types::detection::{DetectedIssue, IssueCategory, Severity};
pub use crate::types::health::{HealthDimensions, HealthScore, HealthTrend};
pub use crate::types::strategy::StrategyResult;
pub use crate::types::token::{CostMetrics, LatencyMetrics, TokenBudget, TokenCount, TokenMetrics};
