use axum::response::sse::Event;
use serde::Serialize;

pub fn json_rpc_event<T: Serialize>(data: &T) -> Result<Event, serde_json::Error> {
    let json = serde_json::to_string(data)?;
    Ok(Event::default().event("message").data(json))
}
