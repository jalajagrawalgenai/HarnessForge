use forge_sdk::types::audit::AuditEvent;

pub struct InteractiveDebugger { events: Vec<AuditEvent>, position: usize, breakpoints: Vec<String> }

impl InteractiveDebugger {
    pub fn new(events: Vec<AuditEvent>) -> Self { Self { events, position: 0, breakpoints: Vec::new() } }
    pub fn add_breakpoint(&mut self, detector: &str) { self.breakpoints.push(detector.into()); }
    pub fn step(&mut self) -> Option<&AuditEvent> {
        if self.position < self.events.len() { self.position += 1; }
        self.current()
    }
    pub fn current(&self) -> Option<&AuditEvent> { self.events.get(self.position) }
    pub fn run_to_breakpoint(&mut self) -> Option<&AuditEvent> {
        while self.position < self.events.len() {
            let ev = &self.events[self.position];
            if self.breakpoints.iter().any(|b| ev.event_type.contains(b)) { return Some(ev); }
            self.position += 1;
        }
        None
    }
    pub fn inspect(&self) -> String {
        match self.current() {
            Some(ev) => format!("[{}] {:?} {} — {}", ev.sequence, ev.phase, ev.event_type, serde_json::to_string(&ev.event_data).unwrap_or_default()),
            None => "End of session".into(),
        }
    }
}
