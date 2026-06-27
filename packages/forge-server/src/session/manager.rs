//! Session manager — sessions are created via the ingest API from REAL
//! agent sessions (Claude Code hooks, LangGraph callbacks, etc.).
//! No MockAgent. No manual session creation. Real agents only.

use crate::session::store::SharedSessionStore;
use tracing;

// Sessions are auto-created by the ingest route when real agent events
// arrive. This module provides helpers for session lifecycle management.
//
// The old MockAgent-based spawn_session has been removed.
// All sessions now come from actual agentic systems via POST /api/v1/ingest/event.

/// Clean up stale sessions (completed/failed older than 24h).
#[allow(dead_code)]
pub async fn cleanup_stale_sessions(store: &SharedSessionStore) {
    let mut sessions = store.write().await;
    let cutoff = chrono::Utc::now() - chrono::Duration::hours(24);
    sessions.retain(|_id, s| {
        if let Some(completed_at) = s.completed_at {
            completed_at > cutoff
        } else {
            true // keep running sessions
        }
    });
    tracing::info!("Cleaned up stale sessions");
}
