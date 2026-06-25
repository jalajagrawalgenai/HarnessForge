use chrono::Utc;
use forge_audit::explainer;
use forge_audit::search::SearchEngine;
use forge_audit::signing;
use forge_audit::trail::AuditTrail;
use forge_sdk::types::audit::{
    AuditEvent, AuditPhase, AuditReport, CheckpointSummary, DetectionSummary, InterventionSummary,
    ObservationSummary,
};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_audit_trail_record() {
    let sid = Uuid::new_v4();
    let mut trail = AuditTrail::new(sid);
    let id = trail.record(
        AuditPhase::Observe,
        "token_usage",
        json!({"tokens": 100}),
        None,
        None,
    );
    assert!(id >= 0);
    assert_eq!(trail.events().len(), 1);
    assert_eq!(trail.query_by_phase(AuditPhase::Observe).len(), 1);
}

#[test]
fn test_audit_trail_multiple_events() {
    let sid = Uuid::new_v4();
    let mut trail = AuditTrail::new(sid);
    trail.record(AuditPhase::Observe, "ev1", json!({}), None, None);
    trail.record(AuditPhase::Detect, "ev2", json!({}), None, None);
    trail.record(AuditPhase::Strategy, "ev3", json!({}), None, None);
    trail.record(AuditPhase::Action, "ev4", json!({}), None, None);
    assert_eq!(trail.events().len(), 4);
    assert_eq!(trail.query_by_phase(AuditPhase::Detect).len(), 1);
}

#[test]
fn test_audit_trail_integrity() {
    let sid = Uuid::new_v4();
    let mut trail = AuditTrail::new(sid);
    trail.record(AuditPhase::Observe, "e1", json!({"a":1}), None, None);
    trail.record(AuditPhase::Observe, "e2", json!({"b":2}), None, None);
    assert!(trail.verify_integrity());
}

#[test]
fn test_audit_trail_clean_integrity() {
    let sid = Uuid::new_v4();
    let mut trail = AuditTrail::new(sid);
    trail.record(AuditPhase::Observe, "e1", json!({"x": 1}), None, None);
    trail.record(AuditPhase::Observe, "e2", json!({"y": 2}), None, None);
    assert!(trail.verify_integrity());
}

#[test]
fn test_search_engine() {
    let sid = Uuid::new_v4();
    let mut trail = AuditTrail::new(sid);
    trail.record(
        AuditPhase::Detect,
        "loop_detected",
        json!({"tool": "read"}),
        None,
        None,
    );
    trail.record(
        AuditPhase::Detect,
        "secret_leak",
        json!({"type": "api_key"}),
        None,
        None,
    );
    let results = SearchEngine::search(trail.events(), "loop");
    assert_eq!(results.len(), 1);
}

#[test]
fn test_signing_standalone() {
    // Test signing module independently
    let events = vec![];
    assert!(signing::verify_integrity(&events, "genesis"));
}

#[test]
fn test_explainer_output() {
    let report = AuditReport {
        session_id: Uuid::new_v4(),
        task: "Build auth".into(),
        agent_type: "solo".into(),
        model: "claude-sonnet".into(),
        duration_secs: 120.0,
        total_tokens: 5000,
        total_cost: 0.05,
        health_score: Some(0.91),
        observations: vec![ObservationSummary {
            dimension: "token".into(),
            event_count: 10,
        }],
        detections: vec![DetectionSummary {
            turn: 5,
            detector: "loop".into(),
            category: "loop".into(),
            severity: "warning".into(),
            confidence: 0.91,
        }],
        interventions: vec![InterventionSummary {
            turn: 5,
            strategy: "nudge".into(),
            outcome: "applied".into(),
            impact: Some("broke loop".into()),
        }],
        checkpoints: vec![CheckpointSummary {
            turn: 5,
            reason: "pre-intervention".into(),
        }],
        harness_effectiveness: Some(0.94),
    };
    let text = explainer::explain(&report);
    assert!(text.contains("FORGE AUDIT REPORT"));
    assert!(text.contains("Build auth"));
    assert!(text.contains("loop"));
}
