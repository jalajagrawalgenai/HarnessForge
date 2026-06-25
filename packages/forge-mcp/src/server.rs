pub struct McpServer {
    port: u16,
}

impl McpServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn expose_resource(&self, uri: &str) -> String {
        format!("forge://{}", uri)
    }
}

pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}
