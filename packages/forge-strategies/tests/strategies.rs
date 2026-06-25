use forge_strategies::nudge::NudgeStrategy;
use forge_strategies::compact::CompactStrategy;
use forge_strategies::circuit_break::CircuitBreakStrategy;
use forge_strategies::pause::PauseStrategy;
use forge_strategies::isolate::IsolateStrategy;
use forge_strategies::diversify::DiversifyStrategy;
use forge_strategies::replace::ReplaceStrategy;
use forge_strategies::degrade::DegradeStrategy;
use forge_strategies::quarantine::QuarantineStrategy;
use forge_strategies::escalate::EscalateStrategy;
use forge_strategies::fork::ForkStrategy;
use forge_strategies::interject::InterjectStrategy;
use forge_strategies::reroute::RerouteStrategy;
use forge_strategies::rollback::RollbackStrategy;
use forge_sdk::traits::strategy::Strategy;
use forge_sdk::types::detection::{DetectedIssue, IssueCategory, Severity};
use uuid::Uuid;

fn make_issue(category: IssueCategory, severity: Severity) -> DetectedIssue {
    DetectedIssue { id: Uuid::new_v4(), agent_id: "test".into(), severity, category,
        description: "test issue".into(), confidence: 0.95,
        suggested_actions: vec!["nudge".into()], evidence_summary: "test".into() }
}

#[tokio::test]
async fn test_nudge_strategy_for_loop() {
    let strategy = NudgeStrategy::new(3);
    let issue = make_issue(IssueCategory::LoopDetected { tool_name: "read".into(), call_count: 5, no_progress_turns: 5 }, Severity::Warning);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().strategy_name, "nudge");
}

#[tokio::test]
async fn test_compact_strategy_for_stale_context() {
    let strategy = CompactStrategy::new(0.6);
    let issue = make_issue(IssueCategory::StaleContext { file_path: "auth.rs".into(), read_count: 4, context_pressure: 0.88 }, Severity::Warning);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
    assert_eq!(result.unwrap().strategy_name, "compact");
}

#[tokio::test]
async fn test_circuit_break_for_secret_leak() {
    let strategy = CircuitBreakStrategy;
    let issue = make_issue(IssueCategory::SecretLeak { secret_type: "api_key".into() }, Severity::Critical);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
}

#[tokio::test]
async fn test_circuit_break_ignores_warning() {
    let strategy = CircuitBreakStrategy;
    let issue = make_issue(IssueCategory::LoopDetected { tool_name: "read".into(), call_count: 3, no_progress_turns: 3 }, Severity::Warning);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_pause_strategy_for_critical() {
    let strategy = PauseStrategy;
    let issue = make_issue(IssueCategory::SecretLeak { secret_type: "key".into() }, Severity::Critical);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
}

#[tokio::test]
async fn test_isolate_strategy_for_error() {
    let strategy = IsolateStrategy;
    let issue = make_issue(IssueCategory::SecretLeak { secret_type: "key".into() }, Severity::Error);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
}

#[tokio::test]
async fn test_diversify_strategy_for_variety_collapse() {
    let strategy = DiversifyStrategy;
    let issue = make_issue(IssueCategory::VarietyCollapse { similarity_score: 0.9, agent_count: 4 }, Severity::Warning);
    let result = strategy.evaluate(&issue).await;
    assert!(result.is_some());
}

#[tokio::test]
async fn test_diversify_ignores_other_categories() {
    let strategy = DiversifyStrategy;
    let issue = make_issue(IssueCategory::LoopDetected { tool_name: "x".into(), call_count: 1, no_progress_turns: 1 }, Severity::Warning);
    assert!(strategy.evaluate(&issue).await.is_none());
}

#[tokio::test]
async fn test_all_strategies_have_unique_names() {
    use std::collections::HashSet;
    let strategies: Vec<Box<dyn Strategy>> = vec![
        Box::new(NudgeStrategy::new(3)), Box::new(CompactStrategy::new(0.6)),
        Box::new(CircuitBreakStrategy), Box::new(PauseStrategy), Box::new(IsolateStrategy),
        Box::new(DiversifyStrategy), Box::new(ReplaceStrategy), Box::new(DegradeStrategy),
        Box::new(QuarantineStrategy), Box::new(EscalateStrategy), Box::new(ForkStrategy),
        Box::new(InterjectStrategy), Box::new(RerouteStrategy), Box::new(RollbackStrategy),
    ];
    let names: HashSet<&str> = strategies.iter().map(|s| s.name()).collect();
    assert_eq!(names.len(), 14, "all strategies must have unique names");
}
