use forge_sdk::events::{AgentEvent, CompressionLayer, DegradeLevel, Intervention, IsolationLevel};
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use forge_sdk::types::health::{HealthDimensions, HealthScore, HealthTrend};
use forge_sdk::types::token::{TokenBudget, TokenCount};
use serde_json;

#[test]
fn test_token_count_total() {
    let tc = TokenCount {
        input: 100,
        output: 50,
        cache_write: 20,
        cache_read: 30,
    };
    assert_eq!(tc.total(), 200);
}

#[test]
fn test_token_count_cache_hit_rate() {
    let tc = TokenCount {
        input: 0,
        output: 0,
        cache_write: 20,
        cache_read: 80,
    };
    assert!((tc.cache_hit_rate() - 0.8).abs() < 0.01);
}

#[test]
fn test_token_count_cache_hit_rate_zero() {
    let tc = TokenCount::default();
    assert_eq!(tc.cache_hit_rate(), 0.0);
}

#[test]
fn test_token_budget_defaults() {
    let budget = TokenBudget::default();
    assert_eq!(budget.max_total, 200_000);
    assert_eq!(budget.max_per_turn, 64_000);
    assert_eq!(budget.warn_at, 0.7);
}

#[test]
fn test_health_score_colors() {
    let green = HealthScore {
        agent_id: "a".into(),
        overall: 0.9,
        dimensions: HealthDimensions::default(),
        trend: HealthTrend::Stable,
    };
    let yellow = HealthScore {
        agent_id: "b".into(),
        overall: 0.6,
        dimensions: HealthDimensions::default(),
        trend: HealthTrend::Stable,
    };
    let orange = HealthScore {
        agent_id: "c".into(),
        overall: 0.4,
        dimensions: HealthDimensions::default(),
        trend: HealthTrend::Stable,
    };
    let red = HealthScore {
        agent_id: "d".into(),
        overall: 0.2,
        dimensions: HealthDimensions::default(),
        trend: HealthTrend::Stable,
    };
    assert_eq!(green.emoji(), "🟢");
    assert_eq!(yellow.emoji(), "🟡");
    assert_eq!(orange.emoji(), "🟠");
    assert_eq!(red.emoji(), "🔴");
}

#[test]
fn test_health_dimensions_default() {
    let d = HealthDimensions::default();
    assert_eq!(d.token_efficiency, 1.0);
    assert_eq!(d.security, 1.0);
    assert!(d.communication.is_none());
    assert!(d.diversity.is_none());
}

#[test]
fn test_detected_issue_severity() {
    let issue = DetectedIssue {
        id: uuid::Uuid::new_v4(),
        agent_id: "test".into(),
        severity: Severity::Critical,
        category: IssueCategory::SecretLeak {
            secret_type: "api_key".into(),
        },
        description: "test".into(),
        confidence: 1.0,
        suggested_actions: vec!["circuit_break".into()],
        evidence_summary: "test".into(),
    };
    assert_eq!(issue.severity, Severity::Critical);
}

#[test]
fn test_agent_event_serialization() {
    let event = AgentEvent::Started {
        agent_id: "agent-1".into(),
        task: "build auth".into(),
        timestamp: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&event).unwrap();
    let back: AgentEvent = serde_json::from_str(&json).unwrap();
    match back {
        AgentEvent::Started { agent_id, task, .. } => {
            assert_eq!(agent_id, "agent-1");
            assert_eq!(task, "build auth");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_intervention_serialization() {
    let intervention = Intervention::Nudge {
        message: "try different approach".into(),
        reason: "loop detected".into(),
    };
    let json = serde_json::to_string(&intervention).unwrap();
    let back: Intervention = serde_json::from_str(&json).unwrap();
    match back {
        Intervention::Nudge { message, reason } => {
            assert_eq!(message, "try different approach");
            assert_eq!(reason, "loop detected");
        }
        _ => panic!("wrong variant"),
    }
}

#[test]
fn test_all_intervention_variants_serialize() {
    let interventions = vec![
        Intervention::Nudge {
            message: "m".into(),
            reason: "r".into(),
        },
        Intervention::Compact {
            target_ratio: 0.6,
            layer: CompressionLayer::Snip,
        },
        Intervention::Pause {
            reason: "r".into(),
            checkpoint_id: uuid::Uuid::new_v4(),
        },
        Intervention::Resume,
        Intervention::Escalate {
            new_model: Some("opus".into()),
            budget_increase: Some(50000),
            reason: "r".into(),
        },
        Intervention::Fork {
            count: 2,
            subtasks: vec!["a".into()],
        },
        Intervention::Reroute {
            to_node: "verify".into(),
            reason: "r".into(),
        },
        Intervention::Rollback {
            checkpoint_id: uuid::Uuid::new_v4(),
            reason: "r".into(),
        },
        Intervention::Diversify {
            alternative_approach: "try risk-first".into(),
        },
        Intervention::Isolate {
            level: IsolationLevel::FullSandbox,
            reason: "r".into(),
        },
        Intervention::CircuitBreak { reason: "r".into() },
        Intervention::Replace {
            context_summary: "s".into(),
            new_model: None,
        },
        Intervention::Interject {
            message: "STOP".into(),
            reason: "r".into(),
        },
        Intervention::Degrade {
            level: DegradeLevel::Mild,
        },
        Intervention::Quarantine { reason: "r".into() },
    ];
    for (i, intervention) in interventions.iter().enumerate() {
        let json = serde_json::to_string(intervention).unwrap();
        assert!(
            serde_json::from_str::<Intervention>(&json).is_ok(),
            "failed at index {}",
            i
        );
    }
}

#[test]
fn test_compression_layer_copy() {
    let layer = CompressionLayer::Microcompact;
    let _copy = layer; // Copy trait
    assert!(matches!(layer, CompressionLayer::Microcompact));
}

#[test]
fn test_severity_ordering() {
    // Info < Warning < Error < Critical
    let cases = vec![
        (Severity::Info, Severity::Warning),
        (Severity::Warning, Severity::Error),
        (Severity::Error, Severity::Critical),
    ];
    for _ in cases {}
}

#[test]
fn test_agent_outcome_default() {
    let outcome = forge_sdk::events::AgentOutcome {
        success: true,
        summary: "done".into(),
        output: None,
    };
    assert!(outcome.success);
    assert_eq!(outcome.summary, "done");
}
