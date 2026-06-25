pub struct TokenCounter;

impl TokenCounter {
    pub fn count(text: &str) -> u64 {
        // Approximate: ~4 chars per token for English text
        (text.len() as f64 / 4.0).ceil() as u64
    }

    pub fn count_messages(messages: &[&str]) -> u64 {
        messages.iter().map(|m| Self::count(m)).sum()
    }
}
