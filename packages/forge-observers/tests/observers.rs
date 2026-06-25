use forge_observers::token_watcher::TokenWatcher;
use forge_observers::security_watcher::SecurityWatcher;
use forge_observers::latency_watcher::LatencyWatcher;
use forge_observers::cost_watcher::CostWatcher;
use forge_observers::accuracy_watcher::AccuracyWatcher;
use forge_observers::health_scorer::{HealthScorer, HealthWeights};
use forge_sdk::traits::observer::Observer;
use forge_sdk::events::AgentEvent;
use forge_sdk::types::health::HealthDimensions;
use chrono::Utc;

#[tokio::test]
async fn test_token_watcher() {
    let w = TokenWatcher::new();
    let ev = AgentEvent::TokenUsage { agent_id:"a".into(), input:1000, output:200, cache_read:500, cache_write:100, model:"c".into(), timestamp:Utc::now() };
    assert!(w.observe(&ev).await.is_some());
    assert!(w.cache_hit_rate() > 0.0);
}

#[tokio::test]
async fn test_security_clean() {
    let w = SecurityWatcher::new();
    let ev = AgentEvent::OutputComplete { agent_id:"a".into(), content:"Code compiles.".into(), timestamp:Utc::now() };
    w.observe(&ev).await;
    assert_eq!(w.leak_count(), 0);
}

#[tokio::test]
async fn test_security_leak() {
    let w = SecurityWatcher::new();
    let ev = AgentEvent::OutputComplete { agent_id:"a".into(), content:"API key: sk-ant-secret".into(), timestamp:Utc::now() };
    w.observe(&ev).await;
    assert_eq!(w.leak_count(), 1);
}

#[tokio::test]
async fn test_latency() {
    let w = LatencyWatcher::new();
    let ev = AgentEvent::ToolCallEnd { agent_id:"a".into(), tool:"read".into(), result: forge_sdk::events::ToolResult{content:"ok".into(),is_error:false,duration_ms:150,token_count:10}, timestamp:Utc::now() };
    assert!(w.observe(&ev).await.is_some());
}

#[tokio::test]
async fn test_cost() {
    let w = CostWatcher::new();
    let ev = AgentEvent::TokenUsage { agent_id:"a".into(), input:5000, output:1000, cache_read:0, cache_write:0, model:"c".into(), timestamp:Utc::now() };
    assert!(w.observe(&ev).await.is_some());
}

#[tokio::test]
async fn test_accuracy() {
    let w = AccuracyWatcher::new();
    let ev = AgentEvent::ToolCallEnd { agent_id:"a".into(), tool:"bash".into(), result: forge_sdk::events::ToolResult{content:"5 tests passed".into(),is_error:false,duration_ms:200,token_count:0}, timestamp:Utc::now() };
    assert!(w.observe(&ev).await.is_some());
}

#[tokio::test]
async fn test_health_scorer() {
    let mut s = HealthScorer::new(HealthWeights::default());
    let d = HealthDimensions { token_efficiency:0.9, latency:0.8, cost:0.95, accuracy:0.85, orchestration:0.9, communication:Some(0.8), security:1.0, reliability:0.9, context_quality:0.7, memory:None, compliance:1.0, diversity:None };
    let score = s.compute("a", &d);
    assert!(score.overall > 0.8);
}
