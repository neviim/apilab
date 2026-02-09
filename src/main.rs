use std::env;
use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;
use tokio::net::TcpListener;

use apilab::session::manager::SessionManager;
use apilab::tools;
use apilab::transport::handler;
use apilab::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host = env::var("MCP_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("MCP_PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("{host}:{port}");

    let state = Arc::new(AppState {
        session_manager: SessionManager::new(),
        tool_registry: Arc::new(tools::build_registry()),
    });

    let app = Router::new()
        .route("/mcp", post(handler::post_mcp))
        .route("/mcp", get(handler::get_mcp))
        .route("/mcp", delete(handler::delete_mcp))
        .with_state(state);

    tracing::info!("MCP server listening on {addr}");
    let listener = TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
