// SecurityWatcher — detects secret leaks, dangerous tools, prompt injection

use async_trait::async_trait;
use std::sync::Mutex;
use forge_sdk::events::AgentEvent;
use forge_sdk::traits::observer::Observer;

pub struct SecurityWatcher {
    secret_leaks: Mutex<u64>,
    dangerous_calls: Mutex<u64>,
    injection_attempts: Mutex<u64>,
}

// Patterns for detection
const SECRET_PATTERNS: &[&str] = &[
    "sk-", "api_key", "BEGIN PRIVATE KEY", "AWS_ACCESS_KEY",
    "ghp_", "xoxb-", "token=", "password=", "secret=",
];

const DANGEROUS_COMMANDS: &[&str] = &[
    "rm -rf /", "curl | bash", "sudo ", "chmod 777",
    "DROP TABLE", "; DROP ", "eval(",
];

const INJECTION_PATTERNS: &[&str] = &[
    "ignore previous instructions", "you are now DAN",
    "system override", "pretend you are",
    "new instructions:", "forget everything",
];

impl SecurityWatcher {
    pub fn new() -> Self {
        Self {
            secret_leaks: Mutex::new(0),
            dangerous_calls: Mutex::new(0),
            injection_attempts: Mutex::new(0),
        }
    }

    fn scan_text(text: &str, patterns: &[&str]) -> Vec<String> {
        let lower = text.to_lowercase();
        patterns
            .iter()
            .filter(|p| lower.contains(&p.to_lowercase()))
            .map(|p| p.to_string())
            .collect()
    }

    pub fn leak_count(&self) -> u64 { *self.secret_leaks.lock().unwrap() }
    pub fn dangerous_count(&self) -> u64 { *self.dangerous_calls.lock().unwrap() }
    pub fn injection_count(&self) -> u64 { *self.injection_attempts.lock().unwrap() }
}

#[async_trait]
impl Observer for SecurityWatcher {
    fn name(&self) -> &'static str { "security" }
    fn dimension(&self) -> &'static str { "security" }

    async fn observe(&self, event: &AgentEvent) -> Option<serde_json::Value> {
        match event {
            AgentEvent::ToolCallStart { tool, args, .. } => {
                let args_str = args.to_string();
                let dangerous = Self::scan_text(&args_str, DANGEROUS_COMMANDS);
                if !dangerous.is_empty() {
                    *self.dangerous_calls.lock().unwrap() += 1;
                    return Some(serde_json::json!({
                        "dimension": "security",
                        "alert": "dangerous_tool_call",
                        "tool": tool,
                        "patterns": dangerous,
                    }));
                }
                None
            }
            AgentEvent::OutputComplete { content, .. } => {
                let leaks = Self::scan_text(content, SECRET_PATTERNS);
                if !leaks.is_empty() {
                    *self.secret_leaks.lock().unwrap() += 1;
                    return Some(serde_json::json!({
                        "dimension": "security",
                        "alert": "secret_leak",
                        "patterns": leaks,
                    }));
                }
                None
            }
            AgentEvent::MessageReceived { content, .. } => {
                if let forge_sdk::events::MessageContent::Text(text) = content {
                    let injections = Self::scan_text(text, INJECTION_PATTERNS);
                    if !injections.is_empty() {
                        *self.injection_attempts.lock().unwrap() += 1;
                        return Some(serde_json::json!({
                            "dimension": "security",
                            "alert": "prompt_injection",
                            "patterns": injections,
                        }));
                    }
                }
                None
            }
            _ => None,
        }
    }
}
