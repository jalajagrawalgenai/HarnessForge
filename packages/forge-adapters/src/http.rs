//! HttpAgent — HTTP API adapter for API-based AI agents.
//!
//! Wraps any OpenAI-compatible or Anthropic-compatible API endpoint.
//! Covers: Solo, OpenAI, Anthropic Claude, GPT, Gemini, BeeAgent, PydanticAI, etc.
//!
//! Supports both OpenAI chat completions format and Anthropic messages format.

use async_trait::async_trait;
use chrono::Utc;
use forge_sdk::agent::{AgentAdapter, AgentType};
use forge_sdk::error::ForgeError;
use forge_sdk::events::{AgentEvent, AgentOutcome, Intervention, ToolResult};
use tokio::sync::mpsc;

/// API format for the HTTP agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiFormat {
    /// OpenAI Chat Completions: POST /v1/chat/completions
    OpenAI,
    /// Anthropic Messages: POST /v1/messages
    Anthropic,
    /// Generic / custom endpoint
    Custom,
}

/// Configuration for an HTTP-based agent.
pub struct HttpAgentConfig {
    /// Base URL of the API (e.g. https://api.anthropic.com)
    pub base_url: String,
    /// API key
    pub api_key: String,
    /// Model name (e.g. claude-sonnet-4-6, gpt-4o)
    pub model: String,
    /// API format
    pub format: ApiFormat,
    /// System prompt
    pub system_prompt: Option<String>,
    /// Max tokens for the response
    pub max_tokens: u32,
    /// Temperature
    pub temperature: f32,
    /// Custom headers
    pub headers: Vec<(String, String)>,
}

/// A real AgentAdapter that wraps any HTTP API-based AI model.
///
/// ```rust
/// use forge_adapters::{HttpAgent, ApiFormat};
/// use forge_sdk::agent::AgentType;
///
/// let agent = HttpAgent::new("claude-1", AgentType::Solo)
///     .base_url("https://api.anthropic.com")
///     .api_key("sk-ant-...")
///     .model("claude-sonnet-4-6")
///     .format(ApiFormat::Anthropic)
///     .system_prompt("You are a helpful coding assistant.");
/// ```
pub struct HttpAgent {
    id: String,
    agent_type: AgentType,
    config: HttpAgentConfig,
    client: reqwest::Client,
}

impl HttpAgent {
    /// Create a new HttpAgent.
    pub fn new(id: impl Into<String>, agent_type: AgentType) -> Self {
        let (base_url, format, model) = match agent_type {
            AgentType::Solo | AgentType::PydanticAI | AgentType::BeeAgent => {
                ("https://api.anthropic.com".into(), ApiFormat::Anthropic, "claude-sonnet-4-6".into())
            }
            _ => {
                ("https://api.anthropic.com".into(), ApiFormat::Anthropic, "claude-sonnet-4-6".into())
            }
        };

        Self {
            id: id.into(),
            agent_type,
            config: HttpAgentConfig {
                base_url,
                api_key: std::env::var("ANTHROPIC_API_KEY").unwrap_or_default(),
                model,
                format,
                system_prompt: None,
                max_tokens: 4096,
                temperature: 0.7,
                headers: vec![],
            },
            client: reqwest::Client::new(),
        }
    }

    /// Set the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = key.into();
        self
    }

    /// Set the model name.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Set the API format.
    pub fn format(mut self, fmt: ApiFormat) -> Self {
        self.config.format = fmt;
        self
    }

    /// Set the system prompt.
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = Some(prompt.into());
        self
    }

    /// Set max tokens.
    pub fn max_tokens(mut self, max: u32) -> Self {
        self.config.max_tokens = max;
        self
    }

    /// Set temperature.
    pub fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = temp;
        self
    }

    /// Add a custom header.
    pub fn header(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.config.headers.push((key.into(), val.into()));
        self
    }

    /// Build the API request body based on format.
    fn build_request_body(&self, task: &str) -> serde_json::Value {
        match self.config.format {
            ApiFormat::OpenAI => {
                let mut messages = vec![];
                if let Some(ref sys) = self.config.system_prompt {
                    messages.push(serde_json::json!({
                        "role": "system",
                        "content": sys
                    }));
                }
                messages.push(serde_json::json!({
                    "role": "user",
                    "content": task
                }));
                serde_json::json!({
                    "model": self.config.model,
                    "messages": messages,
                    "max_tokens": self.config.max_tokens,
                    "temperature": self.config.temperature,
                })
            }
            ApiFormat::Anthropic => {
                let mut body = serde_json::json!({
                    "model": self.config.model,
                    "max_tokens": self.config.max_tokens,
                    "messages": [{"role": "user", "content": task}],
                });
                if let Some(ref sys) = self.config.system_prompt {
                    body["system"] = serde_json::json!(sys);
                }
                body
            }
            ApiFormat::Custom => serde_json::json!({
                "prompt": task,
                "max_tokens": self.config.max_tokens,
            }),
        }
    }

    /// Build the HTTP request.
    fn build_request(&self, task: &str) -> Result<reqwest::Request, ForgeError> {
        let body = self.build_request_body(task);
        let endpoint = match self.config.format {
            ApiFormat::OpenAI => format!("{}/v1/chat/completions", self.config.base_url),
            ApiFormat::Anthropic => format!("{}/v1/messages", self.config.base_url),
            ApiFormat::Custom => format!("{}/v1/completions", self.config.base_url),
        };

        let mut req = self
            .client
            .post(&endpoint)
            .json(&body)
            .build()
            .map_err(|e| ForgeError::ToolExecution(
                format!("Failed to build HTTP request: {}", e)))?;

        // Set auth header
        let auth_header = match self.config.format {
            ApiFormat::Anthropic => ("x-api-key", self.config.api_key.clone()),
            _ => ("Authorization", format!("Bearer {}", self.config.api_key)),
        };
        req.headers_mut().insert(
            reqwest::header::HeaderName::from_bytes(auth_header.0.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(&auth_header.1).unwrap(),
        );

        // Custom headers
        for (k, v) in &self.config.headers {
            if let (Ok(key), Ok(val)) = (
                reqwest::header::HeaderName::from_bytes(k.as_bytes()),
                reqwest::header::HeaderValue::from_str(v),
            ) {
                req.headers_mut().insert(key, val);
            }
        }

        Ok(req)
    }

    /// Extract response text based on API format.
    fn extract_response(&self, body: &serde_json::Value) -> (String, u64) {
        let (text, tokens) = match self.config.format {
            ApiFormat::OpenAI => {
                let text = body["choices"][0]["message"]["content"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let tokens = body["usage"]["total_tokens"].as_u64().unwrap_or(0);
                (text, tokens)
            }
            ApiFormat::Anthropic => {
                let text = body["content"][0]["text"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                let tokens = body["usage"]["input_tokens"].as_u64().unwrap_or(0)
                    + body["usage"]["output_tokens"].as_u64().unwrap_or(0);
                (text, tokens)
            }
            ApiFormat::Custom => {
                let text = body["text"]
                    .as_str()
                    .or_else(|| body["response"].as_str())
                    .unwrap_or("")
                    .to_string();
                (text, 0)
            }
        };
        (text, tokens)
    }

    /// Check for harness interventions.
    fn handle_interventions(
        &self,
        rx: &mut mpsc::Receiver<Intervention>,
    ) -> Result<(), ForgeError> {
        while let Ok(intervention) = rx.try_recv() {
            match intervention {
                Intervention::CircuitBreak { reason } => {
                    return Err(ForgeError::CircuitBroken { reason });
                }
                Intervention::Pause { reason, .. } => {
                    tracing::warn!(agent_id = %self.id, reason = %reason, "Paused by harness");
                }
                _ => {
                    tracing::debug!(agent_id = %self.id, ?intervention, "Intervention received");
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl AgentAdapter for HttpAgent {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    async fn run(
        &mut self,
        task: &str,
        event_tx: mpsc::Sender<AgentEvent>,
        mut intervention_rx: mpsc::Receiver<Intervention>,
    ) -> Result<AgentOutcome, ForgeError> {
        let now = Utc::now();

        // 1. Started
        let _ = event_tx
            .send(AgentEvent::Started {
                agent_id: self.id.clone(),
                task: task.to_string(),
                timestamp: now,
            })
            .await;

        // 2. Pre-intervention check
        self.handle_interventions(&mut intervention_rx)?;

        // 3. Thinking start
        let _ = event_tx
            .send(AgentEvent::ThinkingStart {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 4. Build and send HTTP request
        let req = self.build_request(task)?;
        let start = std::time::Instant::now();

        let response = self.client.execute(req).await.map_err(|e| {
            ForgeError::ToolExecution(
                format!("HTTP request failed: {}", e))
        })?;

        let status = response.status();
        let response_body: serde_json::Value =
            response.json().await.unwrap_or(serde_json::Value::Null);
        let duration_ms = start.elapsed().as_millis() as u64;

        let is_error = !status.is_success();
        let (text, token_count) = if is_error {
            (
                response_body["error"]["message"]
                    .as_str()
                    .unwrap_or("Unknown API error")
                    .to_string(),
                0,
            )
        } else {
            self.extract_response(&response_body)
        };

        // 5. Tool call start (the "tool" is the model API itself)
        let _ = event_tx
            .send(AgentEvent::ToolCallStart {
                agent_id: self.id.clone(),
                tool: self.config.model.clone(),
                args: serde_json::json!({"format": format!("{:?}", self.config.format), "task": task}),
                timestamp: Utc::now(),
            })
            .await;

        // 6. Tool call end
        let _ = event_tx
            .send(AgentEvent::ToolCallEnd {
                agent_id: self.id.clone(),
                tool: self.config.model.clone(),
                result: ToolResult {
                    content: text.clone(),
                    is_error,
                    duration_ms,
                    token_count,
                },
                timestamp: Utc::now(),
            })
            .await;

        // 7. Post-intervention check
        self.handle_interventions(&mut intervention_rx)?;

        // 8. Thinking end
        let _ = event_tx
            .send(AgentEvent::ThinkingEnd {
                agent_id: self.id.clone(),
                timestamp: Utc::now(),
            })
            .await;

        // 9. Completed
        let _ = event_tx
            .send(AgentEvent::Completed {
                agent_id: self.id.clone(),
                summary: text.chars().take(300).collect(),
                timestamp: Utc::now(),
            })
            .await;

        Ok(AgentOutcome {
            success: !is_error,
            summary: format!(
                "{} via {} completed in {}ms ({} tokens)",
                self.config.model,
                self.config.base_url,
                duration_ms,
                token_count
            ),
            output: Some(serde_json::json!({
                "model": self.config.model,
                "duration_ms": duration_ms,
                "token_count": token_count,
                "status_code": status.as_u16(),
                "response": text,
            }).to_string()),
        })
    }
}
