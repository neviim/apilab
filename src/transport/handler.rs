use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::Json;
use futures_util::stream;

use uuid::Uuid;

use crate::gateway::router::{dispatch, DispatchResult};
use crate::protocol::error::{self, error_object};
use crate::protocol::jsonrpc::{JsonRpcErrorResponse, JsonRpcRequest};
use crate::session::state::Session;
use crate::AppState;

pub async fn post_mcp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> Response {
    // Validate JSON-RPC version
    if req.jsonrpc != "2.0" {
        let err = JsonRpcErrorResponse::new(
            req.id.clone().unwrap_or(serde_json::Value::Null),
            error_object(error::INVALID_REQUEST, "Invalid JSON-RPC version"),
        );
        return (StatusCode::OK, Json(err)).into_response();
    }

    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let is_initialize = req.method == "initialize";

    // Validate session exists (except for initialize)
    if !is_initialize {
        match &session_id {
            None => {
                let err = JsonRpcErrorResponse::new(
                    req.id.clone().unwrap_or(serde_json::Value::Null),
                    error_object(error::INVALID_REQUEST, "Missing Mcp-Session-Id header"),
                );
                return (StatusCode::BAD_REQUEST, Json(err)).into_response();
            }
            Some(sid) => {
                let exists = state.session_manager.with(sid, |_| ()).is_some();
                if !exists {
                    let err = JsonRpcErrorResponse::new(
                        req.id.clone().unwrap_or(serde_json::Value::Null),
                        error_object(error::INVALID_REQUEST, "Unknown session"),
                    );
                    return (StatusCode::NOT_FOUND, Json(err)).into_response();
                }
                // Touch session
                state.session_manager.with_mut(sid, |s| s.touch());
            }
        }
    }

    let result = dispatch(
        &req.method,
        req.id,
        req.params,
        session_id.as_deref(),
        &state.session_manager,
        &state.tool_registry,
    )
    .await;

    match result {
        DispatchResult::Response(resp) => {
            if is_initialize {
                // Create session and return Mcp-Session-Id header
                let sid = Uuid::new_v4().to_string();
                let protocol_version = resp
                    .result
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("2025-06-18")
                    .to_string();
                let session = Session::new(sid.clone(), protocol_version);
                state.session_manager.create(session);

                (
                    StatusCode::OK,
                    [("mcp-session-id", sid)],
                    Json(resp),
                )
                    .into_response()
            } else {
                (StatusCode::OK, Json(resp)).into_response()
            }
        }
        DispatchResult::Accepted => StatusCode::ACCEPTED.into_response(),
        DispatchResult::Error(err) => (StatusCode::OK, Json(err)).into_response(),
    }
}

pub async fn get_mcp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Response {
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok());

    match session_id {
        None => StatusCode::BAD_REQUEST.into_response(),
        Some(sid) => {
            let exists = state.session_manager.with(sid, |_| ()).is_some();
            if !exists {
                return StatusCode::NOT_FOUND.into_response();
            }

            // Phase 1: just a keep-alive stream (no server-initiated messages yet)
            let stream = stream::pending::<Result<Event, Infallible>>();
            Sse::new(stream)
                .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
                .into_response()
        }
    }
}

pub async fn delete_mcp(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> StatusCode {
    let session_id = headers
        .get("mcp-session-id")
        .and_then(|v| v.to_str().ok());

    match session_id {
        None => StatusCode::BAD_REQUEST,
        Some(sid) => {
            if state.session_manager.destroy(sid) {
                StatusCode::OK
            } else {
                StatusCode::NOT_FOUND
            }
        }
    }
}
