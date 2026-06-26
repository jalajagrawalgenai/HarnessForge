//! Forge Export — Native observability platform exporters.
//!
//! Direct, native integration with:
//! - LangFuse (traces, observations, scores)
//! - W&B Weave (calls, traces, feedback)
//! - OpenTelemetry (spans, metrics, logs)
//! - PagerDuty / OpsGenie (incidents, alerts)

use serde::{Deserialize, Serialize};

/// Export target platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExportTarget {
    LangFuse,
    WandB,
    OpenTelemetry,
    PagerDuty,
    OpsGenie,
    Splunk,
    ElasticSearch,
    Datadog,
    Slack,
    Discord,
}

/// Configuration for exporting Forge sessions to external platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConfig {
    pub target: ExportTarget,
    pub enabled: bool,
    pub endpoint: String,
    pub api_key: String,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
}

/// A Forge session event formatted for external export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableEvent {
    pub session_id: String,
    pub agent_id: String,
    pub agent_type: String,
    pub timestamp: String,
    pub phase: String,
    pub event_type: String,
    pub dimension: Option<String>,
    pub severity: Option<String>,
    pub detector: Option<String>,
    pub strategy: Option<String>,
    pub health_score: Option<f64>,
    pub data: serde_json::Value,
}

/// LangFuse trace format.
pub mod langfuse {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct LangFuseTrace {
        pub name: String,
        pub user_id: String,
        pub session_id: String,
        pub metadata: serde_json::Value,
        pub tags: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    pub struct LangFuseObservation {
        pub trace_id: String,
        pub name: String,
        pub r#type: String,
        pub start_time: String,
        pub end_time: String,
        pub metadata: serde_json::Value,
    }

    /// Convert a Forge detection to a LangFuse score.
    pub fn detection_to_score(
        trace_id: &str,
        detector: &str,
        severity: &str,
        confidence: f64,
    ) -> serde_json::Value {
        serde_json::json!({
            "trace_id": trace_id,
            "name": format!("forge.{}", detector),
            "value": confidence,
            "comment": format!("{} detected ({})", detector, severity),
        })
    }

    /// Build a LangFuse trace from a Forge session.
    pub fn session_to_trace(session_id: &str, task: &str, agent_type: &str) -> LangFuseTrace {
        LangFuseTrace {
            name: task.chars().take(100).collect(),
            user_id: "forge-harness".into(),
            session_id: session_id.into(),
            metadata: serde_json::json!({
                "agent_type": agent_type,
                "source": "forge",
            }),
            tags: vec!["forge".into(), agent_type.into()],
        }
    }
}

/// W&B Weave format.
pub mod wandb {
    use super::*;

    #[derive(Debug, Serialize)]
    pub struct WeaveCall {
        pub op_name: String,
        pub trace_id: String,
        pub parent_id: Option<String>,
        pub started_at: String,
        pub ended_at: Option<String>,
        pub inputs: serde_json::Value,
        pub outputs: Option<serde_json::Value>,
        pub attributes: serde_json::Value,
    }

    /// Create a Weave call from a Forge agent event.
    pub fn event_to_call(event: &ExportableEvent) -> WeaveCall {
        WeaveCall {
            op_name: format!("forge.{}", event.event_type),
            trace_id: event.session_id.clone(),
            parent_id: None,
            started_at: event.timestamp.clone(),
            ended_at: Some(event.timestamp.clone()),
            inputs: serde_json::json!({"agent_id": event.agent_id}),
            outputs: Some(event.data.clone()),
            attributes: serde_json::json!({
                "phase": event.phase,
                "health_score": event.health_score,
                "dimension": event.dimension,
            }),
        }
    }
}

/// PagerDuty / OpsGenie incident creation.
pub mod alerting {
    use super::*;

    /// PagerDuty incident severity.
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    pub enum PdSeverity {
        #[serde(rename = "critical")]
        Critical,
        #[serde(rename = "error")]
        Error,
        #[serde(rename = "warning")]
        Warning,
        #[serde(rename = "info")]
        Info,
    }

    /// PagerDuty incident payload (Events API v2).
    #[derive(Debug, Serialize)]
    pub struct PagerDutyIncident {
        pub routing_key: String,
        pub event_action: String, // "trigger", "acknowledge", "resolve"
        pub dedup_key: String,
        pub payload: PagerDutyPayload,
    }

    #[derive(Debug, Serialize)]
    pub struct PagerDutyPayload {
        pub summary: String,
        pub source: String,
        pub severity: PdSeverity,
        pub custom_details: serde_json::Value,
    }

    /// Create a PagerDuty incident from a Forge detection.
    pub fn detection_to_incident(
        routing_key: &str,
        session_id: &str,
        detector: &str,
        severity: &str,
        evidence: &str,
    ) -> PagerDutyIncident {
        let pd_severity = match severity {
            "critical" => PdSeverity::Critical,
            "error" => PdSeverity::Error,
            "warning" => PdSeverity::Warning,
            _ => PdSeverity::Info,
        };

        PagerDutyIncident {
            routing_key: routing_key.into(),
            event_action: "trigger".into(),
            dedup_key: format!("forge-{}-{}", session_id, detector),
            payload: PagerDutyPayload {
                summary: format!("Forge: {} detected in session {}", detector, session_id),
                source: "forge-harness".into(),
                severity: pd_severity,
                custom_details: serde_json::json!({
                    "session_id": session_id,
                    "detector": detector,
                    "severity": severity,
                    "evidence": evidence,
                }),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_langfuse_detection_to_score() {
        let score = langfuse::detection_to_score("trace-1", "loop", "warning", 0.87);
        assert_eq!(score["name"], "forge.loop");
        assert_eq!(score["value"], 0.87);
    }

    #[test]
    fn test_pagerduty_incident_creation() {
        let incident = alerting::detection_to_incident(
            "rk-test",
            "session-1",
            "secret_leak",
            "critical",
            "API key found in output",
        );
        assert_eq!(incident.event_action, "trigger");
        assert_eq!(incident.payload.severity, alerting::PdSeverity::Critical);
        assert!(incident.payload.summary.contains("secret_leak"));
    }

    #[test]
    fn test_wandb_call_from_event() {
        let event = ExportableEvent {
            session_id: "s1".into(),
            agent_id: "a1".into(),
            agent_type: "solo".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            phase: "detect".into(),
            event_type: "loop_detected".into(),
            dimension: Some("orchestration".into()),
            severity: Some("warning".into()),
            detector: Some("loop".into()),
            strategy: None,
            health_score: Some(0.85),
            data: serde_json::json!({"count": 4}),
        };
        let call = wandb::event_to_call(&event);
        assert_eq!(call.op_name, "forge.loop_detected");
    }
}
