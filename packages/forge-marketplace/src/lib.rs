//! Forge Marketplace — Community plugin registry client.
//!
//! Browse, install, publish, and rate community-contributed:
//! - Custom observers
//! - Custom detectors
//! - Custom strategies
//! - Skills (bundled harness configs)
//!
//! Registry URL: https://marketplace.forge.dev (planned)

use serde::{Deserialize, Serialize};

/// A plugin in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub plugin_type: PluginType,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub rating: f64,
    pub verified: bool,
    pub created_at: String,
    pub updated_at: String,
    pub repository: Option<String>,
}

/// Type of marketplace plugin.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginType {
    Observer,
    Detector,
    Strategy,
    Skill,
    Preset,
    Adapter,
}

impl std::fmt::Display for PluginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Observer => write!(f, "observer"),
            Self::Detector => write!(f, "detector"),
            Self::Strategy => write!(f, "strategy"),
            Self::Skill => write!(f, "skill"),
            Self::Preset => write!(f, "preset"),
            Self::Adapter => write!(f, "adapter"),
        }
    }
}

/// Client for the Forge plugin marketplace.
pub struct MarketplaceClient {
    registry_url: String,
    client: reqwest::Client,
}

impl MarketplaceClient {
    pub fn new(registry_url: impl Into<String>) -> Self {
        Self {
            registry_url: registry_url.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Search for plugins.
    pub async fn search(&self, query: &str, plugin_type: Option<PluginType>) -> Result<Vec<Plugin>, MarketplaceError> {
        let url = format!("{}/api/v1/plugins/search", self.registry_url);
        let mut params = vec![("q", query.to_string())];
        if let Some(pt) = plugin_type {
            params.push(("type", pt.to_string()));
        }
        let resp = self.client.get(&url).query(&params).send().await?;
        Ok(resp.json().await?)
    }

    /// Get a specific plugin by name.
    pub async fn get(&self, name: &str) -> Result<Plugin, MarketplaceError> {
        let url = format!("{}/api/v1/plugins/{}", self.registry_url, name);
        let resp = self.client.get(&url).send().await?;
        Ok(resp.json().await?)
    }

    /// List installed plugins (from local registry).
    pub async fn list_installed(&self) -> Result<Vec<Plugin>, MarketplaceError> {
        // In production: read from ~/.forge/plugins/
        Ok(vec![])
    }

    /// Install a plugin from the marketplace.
    pub async fn install(&self, name: &str, version: Option<&str>) -> Result<(), MarketplaceError> {
        let version = version.unwrap_or("latest");
        let url = format!("{}/api/v1/plugins/{}/versions/{}/download", self.registry_url, name, version);
        let _resp = self.client.get(&url).send().await?;
        Ok(())
    }

    /// Publish a plugin to the marketplace.
    pub async fn publish(&self, _plugin: &Plugin, _api_key: &str) -> Result<(), MarketplaceError> {
        // In production: POST to /api/v1/plugins with plugin metadata + archive
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MarketplaceError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Plugin not found: {0}")]
    NotFound(String),
    #[error("Unauthorized — provide a valid API key")]
    Unauthorized,
}
