use crate::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use forge_sdk::events::Intervention;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            if let Ok(cmd) = serde_json::from_str::<Value>(&text) {
                let action = cmd.get("action").and_then(|v| v.as_str());
                let session_id = cmd
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                match (action, session_id) {
                    (Some("subscribe"), Some(sid)) => {
                        let _ = socket
                            .send(Message::Text(
                                json!({"event":"subscribed","session_id":sid})
                                    .to_string()
                                    .into(),
                            ))
                            .await;

                        let mut rx = {
                            let sessions = state.store.read().await;
                            sessions.get(&sid).map(|s| s.event_broadcaster.subscribe())
                        };

                        if let Some(mut event_rx) = rx.take() {
                            loop {
                                tokio::select! {
                                    event = event_rx.recv() => {
                                        match event {
                                            Ok(agent_event) => {
                                                let json_str = serde_json::to_string(&agent_event).unwrap_or_default();
                                                if socket.send(Message::Text(json_str.into())).await.is_err() { break; }
                                            }
                                            Err(broadcast::error::RecvError::Lagged(n)) => {
                                                let _ = socket.send(Message::Text(
                                                    json!({"event":"lagged","skipped":n}).to_string().into()
                                                )).await;
                                            }
                                            Err(broadcast::error::RecvError::Closed) => break,
                                        }
                                    }
                                    msg = socket.recv() => {
                                        match msg {
                                            Some(Ok(Message::Text(text))) => {
                                                if let Ok(cmd) = serde_json::from_str::<Value>(&text) {
                                                    match cmd.get("action").and_then(|v|v.as_str()) {
                                                        Some("pause") => {
                                                            let sessions = state.store.read().await;
                                                            if let Some(s) = sessions.get(&sid) {
                                                                let _ = s.intervention_tx.try_send(Intervention::Pause {
                                                                    reason:"WebSocket pause".into(),
                                                                    checkpoint_id: uuid::Uuid::new_v4(),
                                                                });
                                                            }
                                                        }
                                                        Some("resume") => {
                                                            let sessions = state.store.read().await;
                                                            if let Some(s) = sessions.get(&sid) {
                                                                let _ = s.intervention_tx.try_send(Intervention::Resume);
                                                            }
                                                        }
                                                        Some("unsubscribe") => break,
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            _ => break,
                                        }
                                    }
                                }
                            }
                        }
                        break;
                    }
                    _ => {
                        let _ = socket.send(Message::Text(
                            json!({"event":"error","message":"send action:subscribe with session_id"}).to_string().into()
                        )).await;
                    }
                }
            }
        }
    }
}
