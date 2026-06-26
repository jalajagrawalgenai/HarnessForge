use axum::extract::Path;
use axum::Json;
use forge_mcp::client::{McpClient, McpServerConfig};
use forge_mcp::discovery::McpDiscovery;

use forge_mcp::server::McpServer;
use serde_json::{json, Value};
use std::sync::Mutex;

static MCP_CLIENT: std::sync::LazyLock<Mutex<McpClient>> =
    std::sync::LazyLock::new(|| Mutex::new(McpClient::new()));

pub async fn list_servers() -> Json<Value> {
    let client = MCP_CLIENT.lock().unwrap();
    let servers: Vec<Value> = client
        .servers()
        .iter()
        .map(|s| json!({"name":s.name,"transport":s.transport,"endpoint":s.endpoint}))
        .collect();
    Json(json!({"servers":servers,"count":client.count()}))
}
pub async fn add_server(Json(body): Json<Value>) -> Json<Value> {
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("unnamed");
    let transport = body
        .get("transport")
        .and_then(|v| v.as_str())
        .unwrap_or("stdio");
    let endpoint = body.get("endpoint").and_then(|v| v.as_str()).unwrap_or("");
    let mut client = MCP_CLIENT.lock().unwrap();
    client.add_server(McpServerConfig {
        name: name.into(),
        transport: transport.into(),
        endpoint: endpoint.into(),
    });
    Json(json!({"added":name,"count":client.count()}))
}
pub async fn remove_server(Path(name): Path<String>) -> Json<Value> {
    Json(json!({"removed":name}))
}
pub async fn discover() -> Json<Value> {
    let servers: Vec<Value> = McpDiscovery::scan()
        .iter()
        .map(|s| json!({"name":s.name,"transport":s.transport,"endpoint":s.endpoint}))
        .collect();
    Json(json!({"discovered":servers,"count":servers.len()}))
}
pub async fn gateway_status() -> Json<Value> {
    Json(json!({"gateway":"not_started","port":0,"upstream_count":0}))
}
pub async fn start_gateway(Json(body): Json<Value>) -> Json<Value> {
    let port = body.get("port").and_then(|v| v.as_u64()).unwrap_or(9100) as u16;
    Json(json!({"gateway":"started","port":port}))
}
pub async fn forge_server_status() -> Json<Value> {
    Json(json!({"mcp_server":"not_started","port":0}))
}
pub async fn start_forge_server(Json(body): Json<Value>) -> Json<Value> {
    let port = body.get("port").and_then(|v| v.as_u64()).unwrap_or(9100) as u16;
    let server = McpServer::new(port);
    Json(json!({"mcp_server":"started","port":server.port()}))
}
pub async fn list_tools() -> Json<Value> {
    Json(json!({"tools":[
        {"name":"forge.run_agent","description":"Start agent with Forge harness"},
        {"name":"forge.pause_session","description":"Pause a running session"},
        {"name":"forge.resume_session","description":"Resume a paused session"},
        {"name":"forge.explain_session","description":"Get audit report"}
    ]}))
}
pub async fn list_resources() -> Json<Value> {
    Json(json!({"resources":[
        {"uri":"forge://sessions","description":"List active sessions"},
        {"uri":"forge://sessions/{id}/health","description":"Health score"},
        {"uri":"forge://sessions/{id}/audit","description":"Audit trail"},
        {"uri":"forge://sessions/{id}/interventions","description":"Intervention history"},
        {"uri":"forge://detectors","description":"Available detectors"},
        {"uri":"forge://strategies","description":"Available strategies"}
    ]}))
}
