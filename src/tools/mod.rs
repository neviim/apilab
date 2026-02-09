pub mod registry;
pub mod ping;

use registry::ToolRegistry;

pub fn build_registry() -> ToolRegistry {
    let mut registry = ToolRegistry::new();
    ping::register(&mut registry);
    registry
}
