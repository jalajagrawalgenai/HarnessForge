// forge-harness/src/event_bus.rs — mpsc-based event bus

use std::sync::Arc;
use tokio::sync::mpsc;
use forge_sdk::events::AgentEvent;

/// The event bus fans out agent events to all registered observers.
/// It also provides an interceptor for injecting interventions back.
pub struct EventBus {
    /// Observers registered per dimension
    observers: Vec<Arc<dyn EventObserver>>,
    /// Recent events buffer for detectors (sliding window)
    recent_events: Vec<AgentEvent>,
    max_recent: usize,
}

pub trait EventObserver: Send + Sync {
    fn name(&self) -> &'static str;
    fn dimension(&self) -> &'static str;
    fn on_event(&self, event: &AgentEvent);
}

impl EventBus {
    pub fn new(max_recent: usize) -> Self {
        Self {
            observers: Vec::new(),
            recent_events: Vec::with_capacity(max_recent),
            max_recent,
        }
    }

    pub fn register(&mut self, observer: Arc<dyn EventObserver>) {
        self.observers.push(observer);
    }

    /// Fan out an event to all observers
    pub fn dispatch(&mut self, event: &AgentEvent) {
        for observer in &self.observers {
            observer.on_event(event);
        }
        self.record(event.clone());
    }

    pub fn dispatch_batch(&mut self, events: &[AgentEvent]) {
        for event in events {
            self.dispatch(event);
        }
    }

    fn record(&mut self, event: AgentEvent) {
        if self.recent_events.len() >= self.max_recent {
            self.recent_events.remove(0);
        }
        self.recent_events.push(event);
    }

    pub fn recent(&self, n: usize) -> &[AgentEvent] {
        let start = self.recent_events.len().saturating_sub(n);
        &self.recent_events[start..]
    }

    pub fn all_recent(&self) -> &[AgentEvent] {
        &self.recent_events
    }
}

/// Create a channel pair for agent events
pub fn agent_channel(capacity: usize) -> (mpsc::Sender<AgentEvent>, mpsc::Receiver<AgentEvent>) {
    mpsc::channel(capacity)
}

/// Create a channel pair for harness interventions
pub fn intervention_channel(
    capacity: usize,
) -> (
    mpsc::Sender<forge_sdk::events::Intervention>,
    mpsc::Receiver<forge_sdk::events::Intervention>,
) {
    mpsc::channel(capacity)
}
