use serde_json::Value;

use crate::protocol::error::{self, error_object};
use crate::protocol::jsonrpc::{JsonRpcErrorResponse, JsonRpcResponse};
use crate::protocol::mcp::{
    InitializeParams, InitializeResult, ServerCapabilities, ServerInfo,
    ToolCallParams, ToolsCapability, ToolsListResult,
};
use crate::session::manager::SessionManager;
use crate::tools::registry::ToolRegistry;

pub enum DispatchResult {
    Response(JsonRpcResponse),
    Accepted,
    Error(JsonRpcErrorResponse),
}

pub async fn dispatch(
    method: &str,
    id: Option<Value>,
    params: Option<Value>,
    session_id: Option<&str>,
    sessions: &SessionManager,
    tools: &ToolRegistry,
) -> DispatchResult {
    match method {
        "initialize" => handle_initialize(id, params, sessions),
        "notifications/initialized" => handle_initialized(session_id, sessions),
        "ping" => handle_ping(id),
        "tools/list" => handle_tools_list(id, tools),
        "tools/call" => handle_tools_call(id, params, tools).await,
        _ => {
            let req_id = id.unwrap_or(Value::Null);
            DispatchResult::Error(JsonRpcErrorResponse::new(
                req_id,
                error_object(error::METHOD_NOT_FOUND, "Method not found"),
            ))
        }
    }
}

fn handle_initialize(
    id: Option<Value>,
    params: Option<Value>,
    _sessions: &SessionManager,
) -> DispatchResult {
    let req_id = id.unwrap_or(Value::Null);

    let init_params: InitializeParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(v) => v,
            Err(_) => {
                return DispatchResult::Error(JsonRpcErrorResponse::new(
                    req_id,
                    error_object(error::INVALID_PARAMS, "Invalid initialize params"),
                ));
            }
        },
        None => {
            return DispatchResult::Error(JsonRpcErrorResponse::new(
                req_id,
                error_object(error::INVALID_PARAMS, "Missing params"),
            ));
        }
    };

    let result = InitializeResult {
        protocol_version: init_params.protocol_version,
        capabilities: ServerCapabilities {
            tools: Some(ToolsCapability {}),
        },
        server_info: ServerInfo {
            name: "apilab".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    match serde_json::to_value(result) {
        Ok(v) => DispatchResult::Response(JsonRpcResponse::new(req_id, v)),
        Err(_) => DispatchResult::Error(JsonRpcErrorResponse::new(
            req_id,
            error_object(error::INTERNAL_ERROR, "Serialization failed"),
        )),
    }
}

fn handle_initialized(session_id: Option<&str>, sessions: &SessionManager) -> DispatchResult {
    if let Some(sid) = session_id {
        sessions.with_mut(sid, |s| {
            s.initialized = true;
            s.touch();
        });
    }
    DispatchResult::Accepted
}

fn handle_ping(id: Option<Value>) -> DispatchResult {
    let req_id = id.unwrap_or(Value::Null);
    DispatchResult::Response(JsonRpcResponse::new(req_id, serde_json::json!({})))
}

fn handle_tools_list(id: Option<Value>, tools: &ToolRegistry) -> DispatchResult {
    let req_id = id.unwrap_or(Value::Null);
    let result = ToolsListResult {
        tools: tools.list(),
    };
    match serde_json::to_value(result) {
        Ok(v) => DispatchResult::Response(JsonRpcResponse::new(req_id, v)),
        Err(_) => DispatchResult::Error(JsonRpcErrorResponse::new(
            req_id,
            error_object(error::INTERNAL_ERROR, "Serialization failed"),
        )),
    }
}

async fn handle_tools_call(
    id: Option<Value>,
    params: Option<Value>,
    tools: &ToolRegistry,
) -> DispatchResult {
    let req_id = id.unwrap_or(Value::Null);

    let call_params: ToolCallParams = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(v) => v,
            Err(_) => {
                return DispatchResult::Error(JsonRpcErrorResponse::new(
                    req_id,
                    error_object(error::INVALID_PARAMS, "Invalid tool call params"),
                ));
            }
        },
        None => {
            return DispatchResult::Error(JsonRpcErrorResponse::new(
                req_id,
                error_object(error::INVALID_PARAMS, "Missing params"),
            ));
        }
    };

    match tools.call(&call_params.name, call_params.arguments).await {
        Some(result) => match serde_json::to_value(result) {
            Ok(v) => DispatchResult::Response(JsonRpcResponse::new(req_id, v)),
            Err(_) => DispatchResult::Error(JsonRpcErrorResponse::new(
                req_id,
                error_object(error::INTERNAL_ERROR, "Serialization failed"),
            )),
        },
        None => DispatchResult::Error(JsonRpcErrorResponse::new(
            req_id,
            error_object(error::METHOD_NOT_FOUND, &format!("Tool '{}' not found", call_params.name)),
        )),
    }
}
