use forge_harness::event_bus::{EventBus, EventObserver};
use forge_harness::plugin_registry::PluginRegistry;
use forge_harness::checkpoint::CheckpointManager;
use forge_harness::human_gate::{HumanGate, HumanGateConfig, GateState};
use forge_harness::config::ForgeConfig;
use forge_sdk::events::AgentEvent;
use chrono::Utc;
use serde_json::json;
use uuid::Uuid;
use std::sync::Arc;

struct TestObserver;
impl EventObserver for TestObserver {
    fn name(&self) -> &'static str { "test" }
    fn dimension(&self) -> &'static str { "test" }
    fn on_event(&self, _e: &AgentEvent) {}
}

#[test]
fn test_event_bus_dispatch() {
    let mut bus = EventBus::new(100);
    bus.register(Arc::new(TestObserver));
    let ev = AgentEvent::ThinkingStart { agent_id: "a".into(), timestamp: Utc::now() };
    bus.dispatch(&ev);
    assert_eq!(bus.all_recent().len(), 1);
}

#[test]
fn test_event_bus_window() {
    let mut bus = EventBus::new(3);
    for i in 0..5 {
        bus.dispatch(&AgentEvent::ThinkingStart { agent_id: i.to_string(), timestamp: Utc::now() });
    }
    assert_eq!(bus.all_recent().len(), 3);
}

#[test]
fn test_plugin_registry_empty() {
    let reg = PluginRegistry::new();
    assert!(reg.strategies().is_empty());
}

#[test]
fn test_checkpoint_save_retrieve() {
    let mut cm = CheckpointManager::new(5);
    let cp = cm.save(Uuid::new_v4(), 1, json!({"s":"ok"}), json!({"t":100}), None, None);
    assert!(cm.latest().is_some());
    assert_eq!(cp.id, cm.latest().unwrap().id);
}

#[test]
fn test_checkpoint_eviction() {
    let mut cm = CheckpointManager::new(2);
    let sid = Uuid::new_v4();
    cm.save(sid, 1, json!({}), json!({}), None, None);
    cm.save(sid, 2, json!({}), json!({}), None, None);
    cm.save(sid, 3, json!({}), json!({}), None, None);
    assert_eq!(cm.list().len(), 2);
}

#[test]
fn test_human_gate_critical_pauses() {
    let gate = HumanGate::new(HumanGateConfig::default());
    assert!(gate.should_pause(&forge_sdk::types::detection::Severity::Critical, 1.0, 1.0));
}

#[test]
fn test_human_gate_normal_no_pause() {
    let gate = HumanGate::new(HumanGateConfig::default());
    assert!(!gate.should_pause(&forge_sdk::types::detection::Severity::Warning, 1.0, 0.9));
}

#[test]
fn test_human_gate_state_flow() {
    let mut gate = HumanGate::new(HumanGateConfig::default());
    assert_eq!(*gate.state(), GateState::Open);
    gate.pause("t".into());
    assert_eq!(*gate.state(), GateState::Paused);
    gate.approve();
    assert_eq!(*gate.state(), GateState::Approved);
}

#[test]
fn test_config_defaults() {
    let c = ForgeConfig::default();
    assert_eq!(c.models.default, "claude-sonnet-4-6");
    assert_eq!(c.harness.checkpoint_interval, 10);
}
