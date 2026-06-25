use forge_sdk::types::audit::AuditEvent;

pub struct SessionReplay { events: Vec<AuditEvent>, position: usize }

impl SessionReplay {
    pub fn new(events: Vec<AuditEvent>) -> Self { Self { events, position: 0 } }
    pub fn current(&self) -> Option<&AuditEvent> { self.events.get(self.position) }
    pub fn next(&mut self) -> Option<&AuditEvent> { self.position += 1; self.current() }
    pub fn prev(&mut self) -> Option<&AuditEvent> { if self.position > 0 { self.position -= 1; } self.current() }
    pub fn seek(&mut self, pos: usize) -> Option<&AuditEvent> { self.position = pos.min(self.events.len().saturating_sub(1)); self.current() }
    pub fn jump_to_detection(&mut self, detector: &str) -> Option<&AuditEvent> {
        for (i, e) in self.events.iter().enumerate().skip(self.position) {
            if e.event_type == detector { self.position = i; return Some(e); }
        }
        None
    }
    pub fn position(&self) -> usize { self.position }
    pub fn total(&self) -> usize { self.events.len() }
    pub fn progress(&self) -> f64 { if self.events.is_empty() { 0.0 } else { self.position as f64 / self.events.len() as f64 } }
}
