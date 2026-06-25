use crate::client::McpServerConfig;

pub struct McpDiscovery;

impl McpDiscovery {
    pub fn scan() -> Vec<McpServerConfig> { Vec::new() }
    pub fn discover_local() -> Vec<McpServerConfig> { Vec::new() }
}
