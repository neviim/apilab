use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;

use crate::protocol::mcp::{Tool, ToolCallResult};

pub type ToolHandler = Box<
    dyn Fn(Option<Value>) -> Pin<Box<dyn Future<Output = ToolCallResult> + Send>>
        + Send
        + Sync,
>;

pub struct ToolRegistry {
    tools: HashMap<String, (Tool, ToolHandler)>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Tool, handler: ToolHandler) {
        self.tools.insert(tool.name.clone(), (tool, handler));
    }

    pub fn list(&self) -> Vec<Tool> {
        self.tools.values().map(|(t, _)| t.clone()).collect()
    }

    pub async fn call(&self, name: &str, arguments: Option<Value>) -> Option<ToolCallResult> {
        if let Some((_, handler)) = self.tools.get(name) {
            Some(handler(arguments).await)
        } else {
            None
        }
    }
}
