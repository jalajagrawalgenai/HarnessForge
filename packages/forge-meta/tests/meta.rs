use forge_meta::edit_registry::EditRegistry;
use forge_meta::harness_proposer::HarnessProposer;
use forge_meta::scheduler::ImprovementScheduler;
use forge_meta::weakness_miner::WeaknessMiner;
use forge_meta::MetaConfig;
use forge_meta::MetaHarness;
use forge_meta::SessionAudit;

fn make_audits(count: usize, success_rate: f64) -> Vec<SessionAudit> {
    (0..count)
        .map(|i| SessionAudit {
            session_id: format!("session_{}", i),
            agent_type: "solo".into(),
            model: if i % 3 == 0 {
                "claude-sonnet".into()
            } else {
                "claude-haiku".into()
            },
            success: (i as f64) < (count as f64 * success_rate),
            total_tokens: 5000 + (i * 100) as u64,
            total_cost: 0.02 + (i as f64 * 0.001),
            duration_secs: 60.0 + (i as f64 * 2.0),
            detection_count: if i % 5 == 0 { 2 } else { 1 },
            intervention_count: if i % 5 == 0 { 2 } else { 1 },
            failure_patterns: if i % 3 == 0 {
                vec!["loop_detected".into()]
            } else {
                vec![]
            },
            error_messages: if i % 4 == 0 {
                vec!["context overflow".into()]
            } else {
                vec![]
            },
        })
        .collect()
}

#[test]
fn test_weakness_miner_no_data() {
    let miner = WeaknessMiner::new(50);
    let patterns = miner.mine(&[]).unwrap();
    assert!(patterns.is_empty());
}

#[test]
fn test_weakness_miner_finds_patterns() {
    let miner = WeaknessMiner::new(5);
    let audits = make_audits(60, 0.6);
    let patterns = miner.mine(&audits).unwrap();
    assert!(
        !patterns.is_empty(),
        "Should find at least one pattern with 60 sessions"
    );
    // First pattern should have highest severity
    if patterns.len() > 1 {
        assert!(patterns[0].severity_score >= patterns[1].severity_score);
    }
    assert!(patterns
        .iter()
        .any(|p| p.model_family != "all" || p.occurrence_count >= 3));
}

#[test]
fn test_harness_proposer_generates_edits() {
    let miner = WeaknessMiner::new(5);
    let audits = make_audits(60, 0.5);
    let patterns = miner.mine(&audits).unwrap();
    let proposer = HarnessProposer::new();
    let edits = proposer.propose(&patterns, &serde_json::json!({})).unwrap();
    assert!(!edits.is_empty());
    // Each edit should have rationale
    for edit in &edits {
        assert!(!edit.rationale.is_empty());
        assert!(!edit.id.is_empty());
    }
}

#[test]
fn test_edit_registry_versioning() {
    let mut registry = EditRegistry::new();
    assert_eq!(registry.current_version(), "v1.0.0");
    assert_eq!(registry.total_edits(), 0);

    let edit = forge_meta::harness_proposer::HarnessEdit {
        id: "edit_1".into(),
        weakness_id: "pat_1".into(),
        target: forge_meta::harness_proposer::EditTarget::DetectorThreshold {
            detector: "loop".into(),
            parameter: "threshold".into(),
        },
        change: forge_meta::harness_proposer::EditChange::ModifyValue {
            old: serde_json::json!(6),
            new: serde_json::json!(4),
        },
        rationale: "Lower threshold".into(),
        affected_models: vec!["claude".into()],
        min_chars: 30,
    };
    registry.apply(edit).unwrap();
    assert_eq!(registry.current_version(), "v1.1.0");
    assert_eq!(registry.total_edits(), 1);
    assert_eq!(registry.versions().len(), 2);
}

#[test]
fn test_edit_registry_rollback() {
    let mut registry = EditRegistry::new();
    let edit = forge_meta::harness_proposer::HarnessEdit {
        id: "edit_x".into(),
        weakness_id: "pat_x".into(),
        target: forge_meta::harness_proposer::EditTarget::PresetUpdate {
            preset: "solo".into(),
        },
        change: forge_meta::harness_proposer::EditChange::ModifyValue {
            old: serde_json::json!("a"),
            new: serde_json::json!("b"),
        },
        rationale: "test".into(),
        affected_models: vec![],
        min_chars: 10,
    };
    registry.apply(edit).unwrap();
    let rolled = registry.rollback().unwrap();
    assert_eq!(rolled, "v1.0.0");
    assert_eq!(registry.total_edits(), 0);
}

#[test]
fn test_scheduler_triggers() {
    let mut scheduler = ImprovementScheduler::new().with_trigger(10);
    assert!(!scheduler.should_run());
    for _ in 0..10 {
        scheduler.notify_session_completed();
    }
    assert!(scheduler.should_run());
    scheduler.mark_ran();
    assert!(!scheduler.should_run());
}

#[test]
fn test_scheduler_daily_limit() {
    let mut scheduler = ImprovementScheduler::new()
        .with_trigger(3)
        .with_min_sessions(0); // always eligible via min_sessions
                               // Run max times
    for i in 0..4 {
        for _ in 0..3 {
            scheduler.notify_session_completed();
        }
        assert!(scheduler.should_run(), "Run {} should be allowed", i + 1);
        scheduler.mark_ran();
    }
    for _ in 0..3 {
        scheduler.notify_session_completed();
    }
    assert!(
        !scheduler.should_run(),
        "Should hit daily limit after 4 runs"
    );
}

#[test]
fn test_meta_harness_insufficient_sessions() {
    let config = MetaConfig {
        min_sessions: 100,
        ..MetaConfig::default()
    };
    let mut meta = MetaHarness::new(config);
    let audits = make_audits(10, 0.5);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt
        .block_on(meta.improve(&audits, &serde_json::json!({}), &[]))
        .unwrap();
    assert_eq!(result.patterns_found, 0);
}

#[test]
fn test_session_audit_serialization() {
    let audit = SessionAudit {
        session_id: "s1".into(),
        agent_type: "solo".into(),
        model: "claude".into(),
        success: true,
        total_tokens: 1000,
        total_cost: 0.01,
        duration_secs: 30.0,
        detection_count: 1,
        intervention_count: 1,
        failure_patterns: vec!["loop".into()],
        error_messages: vec![],
    };
    let json = serde_json::to_string(&audit).unwrap();
    let back: SessionAudit = serde_json::from_str(&json).unwrap();
    assert_eq!(back.session_id, "s1");
    assert!(back.success);
}
