use axum::extract::ws::{WebSocket, Message, WebSocketUpgrade};
use axum::response::IntoResponse;
use tokio::sync::broadcast;
use serde_json::json;

#[allow(dead_code)]
pub struct WsState { pub tx: broadcast::Sender<String> }

#[allow(dead_code)]
impl WsState {
    pub fn new() -> Self { let (tx, _) = broadcast::channel(256); Self { tx } }
    pub fn broadcast(&self, msg: &str) { let _ = self.tx.send(msg.to_string()); }
}

#[allow(dead_code)]
pub async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

#[allow(dead_code)]
async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            if let Ok(cmd) = serde_json::from_str::<serde_json::Value>(&text) {
                let response = match cmd.get("action").and_then(|v| v.as_str()) {
                    Some("subscribe") => json!({"event":"subscribed","session":cmd.get("session_id")}),
                    Some("pause") => json!({"event":"paused"}),
                    Some("resume") => json!({"event":"resumed"}),
                    _ => json!({"event":"unknown"}),
                };
                let _ = socket.send(Message::Text(response.to_string().into())).await;
            }
        }
    }
}
