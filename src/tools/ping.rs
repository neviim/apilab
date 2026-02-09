use serde_json::json;

use crate::protocol::mcp::{Tool, ToolCallResult, ToolContent};
use super::registry::ToolRegistry;

pub fn register(registry: &mut ToolRegistry) {
    let tool = Tool {
        name: "ping".to_string(),
        description: "Returns pong".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {},
        }),
    };

    registry.register(
        tool,
        Box::new(|_args| {
            Box::pin(async {
                ToolCallResult {
                    content: vec![ToolContent::Text {
                        text: "pong".to_string(),
                    }],
                    is_error: None,
                }
            })
        }),
    );
}
