use forge_mcp::client::{McpClient, McpServerConfig};
use forge_mcp::discovery::McpDiscovery;
use forge_mcp::gateway::McpGateway;
use forge_mcp::server::McpServer;

#[test]
fn test_mcp_client() {
    let mut client = McpClient::new();
    assert_eq!(client.count(), 0);
    client.add_server(McpServerConfig {
        name: "test".into(),
        transport: "stdio".into(),
        endpoint: "http://localhost".into(),
    });
    assert_eq!(client.count(), 1);
    assert_eq!(client.servers().len(), 1);
}

#[test]
fn test_mcp_server() {
    let server = McpServer::new(9100);
    assert_eq!(server.port(), 9100);
    assert!(server.expose_resource("sessions").contains("forge://"));
}

#[test]
fn test_mcp_gateway() {
    let client = McpClient::new();
    let gw = McpGateway::new(client, 9200);
    assert_eq!(gw.port(), 9200);
    assert_eq!(gw.upstream_count(), 0);
}

#[test]
fn test_mcp_discovery() {
    let servers = McpDiscovery::scan();
    assert!(servers.is_empty()); // No servers in test
}
