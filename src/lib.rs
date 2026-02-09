pub mod protocol;
pub mod session;
pub mod tools;
pub mod gateway;
pub mod transport;

use std::sync::Arc;

use session::manager::SessionManager;
use tools::registry::ToolRegistry;

pub struct AppState {
    pub session_manager: SessionManager,
    pub tool_registry: Arc<ToolRegistry>,
}
