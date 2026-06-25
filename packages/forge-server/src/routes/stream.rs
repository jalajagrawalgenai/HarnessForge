use axum::{extract::Path, response::sse::{Event, Sse}, Json};
use futures::stream::{self, Stream};
use serde_json::{json, Value};
use std::convert::Infallible;
use tokio_stream::StreamExt;

pub async fn session_stream(Path(id): Path<String>) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::iter(vec![
        Event::default().data(json!({"session":id,"turn":1,"event":"thinking","content":"Analyzing..."}).to_string()),
        Event::default().data(json!({"session":id,"turn":2,"event":"tool_call","tool":"read"}).to_string()),
        Event::default().data(json!({"session":id,"turn":3,"event":"output","content":"Complete"}).to_string()),
    ]).map(Ok);
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
