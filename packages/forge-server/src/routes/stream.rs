//! SSE (Server-Sent Events) streaming endpoint for live session events.

use crate::AppState;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// GET /v1/sessions/:id/stream
/// Returns an SSE stream of all AgentEvents for the given session.
pub async fn session_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = {
        let sessions = state.store.read().await;
        sessions
            .get(&id)
            .map(|s| s.event_broadcaster.subscribe())
            .unwrap_or_else(|| {
                let (tx, rx) = tokio::sync::broadcast::channel(16);
                drop(tx);
                rx
            })
    };

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}
