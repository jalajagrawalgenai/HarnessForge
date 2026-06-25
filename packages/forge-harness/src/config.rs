// forge-harness/src/config.rs — Configuration loader

use serde::{Deserialize, Serialize};

/// Top-level Forge configuration (forge.toml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeConfig {
    pub harness: HarnessSection,
    pub models: ModelsSection,
    pub audit: AuditSection,
    #[serde(default)]
    pub mcp: McpSection,
    #[serde(default)]
    pub cloud: CloudSection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessSection {
    pub checkpoint_interval: u32,
    pub max_interventions: u32,
    pub dry_run: bool,
    pub observers: Vec<String>,
    pub detectors: Vec<String>,
    pub strategies: Vec<String>,
    pub human_gates: HumanGateToml,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanGateToml {
    pub before_dangerous_tools: bool,
    pub on_cost_spike: f64,
    pub on_accuracy_drop: f64,
    pub auto_resume_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsSection {
    pub default: String,
    pub fallback: String,
    pub max_tokens: u64,
    pub temperature: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSection {
    pub store: String, // "sqlite" or "postgres"
    pub path: String,
    pub retention_days: u32,
    pub cryptographic_signing: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpSection {
    pub enabled: bool,
    pub servers: Vec<McpServerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CloudSection {
    pub provider: Option<String>,
    pub region: Option<String>,
}

impl Default for ForgeConfig {
    fn default() -> Self {
        Self {
            harness: HarnessSection {
                checkpoint_interval: 10,
                max_interventions: 20,
                dry_run: false,
                observers: vec![
                    "token".into(),
                    "latency".into(),
                    "cost".into(),
                    "accuracy".into(),
                    "security".into(),
                ],
                detectors: vec![
                    "loop".into(),
                    "stale_context".into(),
                    "cost_anomaly".into(),
                    "secret_leak".into(),
                ],
                strategies: vec!["nudge".into(), "compact".into(), "pause".into()],
                human_gates: HumanGateToml {
                    before_dangerous_tools: true,
                    on_cost_spike: 5.0,
                    on_accuracy_drop: 0.7,
                    auto_resume_timeout_secs: 1800,
                },
            },
            models: ModelsSection {
                default: "claude-sonnet-4-6".into(),
                fallback: "claude-haiku-4-5".into(),
                max_tokens: 200_000,
                temperature: 0.7,
            },
            audit: AuditSection {
                store: "sqlite".into(),
                path: "./forge-audit.db".into(),
                retention_days: 90,
                cryptographic_signing: false,
            },
            mcp: McpSection::default(),
            cloud: CloudSection::default(),
        }
    }
}

impl ForgeConfig {
    pub fn from_file(path: &str) -> Result<Self, forge_sdk::error::ForgeError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| forge_sdk::error::ForgeError::Config(e.to_string()))?;
        toml::de::from_str(&content)
            .map_err(|e| forge_sdk::error::ForgeError::Config(e.to_string()))
    }

    pub fn to_file(&self, path: &str) -> Result<(), forge_sdk::error::ForgeError> {
        let content = toml::ser::to_string_pretty(self)
            .map_err(|e| forge_sdk::error::ForgeError::Config(e.to_string()))?;
        std::fs::write(path, content)
            .map_err(|e| forge_sdk::error::ForgeError::Config(e.to_string()))
    }
}
