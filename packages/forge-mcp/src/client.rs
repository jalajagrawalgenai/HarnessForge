use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub name: String,
    pub transport: String,
    pub endpoint: String,
}

pub struct McpClient {
    servers: Vec<McpServerConfig>,
}

impl McpClient {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self { Self { servers: Vec::new() } }
    pub fn add_server(&mut self, config: McpServerConfig) { self.servers.push(config); }
    pub fn servers(&self) -> &[McpServerConfig] { &self.servers }
    pub fn count(&self) -> usize { self.servers.len() }
}
