use crate::client::McpClient;

pub struct McpGateway {
    client: McpClient,
    listen_port: u16,
}

impl McpGateway {
    pub fn new(client: McpClient, port: u16) -> Self {
        Self {
            client,
            listen_port: port,
        }
    }
    pub fn port(&self) -> u16 {
        self.listen_port
    }
    pub fn upstream_count(&self) -> usize {
        self.client.count()
    }
}
