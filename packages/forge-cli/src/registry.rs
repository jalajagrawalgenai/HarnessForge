use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEntry { pub name: String, pub version: String, pub description: String, pub downloads: u64, pub verified: bool }

pub struct PluginMarketplace { plugins: Vec<PluginEntry> }

impl PluginMarketplace {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { plugins: vec![
        PluginEntry { name:"custom-security".into(), version:"1.0.0".into(), description:"Additional security patterns".into(), downloads:1200, verified:true },
        PluginEntry { name:"slack-notifier".into(), version:"0.2.0".into(), description:"Slack notification strategy".into(), downloads:890, verified:true },
        PluginEntry { name:"cost-analyzer".into(), version:"1.1.0".into(), description:"Advanced cost analysis observer".into(), downloads:650, verified:false },
    ]}}
    pub fn search(&self, query: &str) -> Vec<&PluginEntry> { self.plugins.iter().filter(|p| p.name.contains(query) || p.description.contains(query)).collect() }
    pub fn list(&self) -> &[PluginEntry] { &self.plugins }
}
