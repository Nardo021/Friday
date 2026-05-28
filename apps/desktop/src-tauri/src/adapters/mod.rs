pub mod cursor_cli_local;
pub mod cursor_cloud_agent;
pub mod cursor_sdk_local;
pub mod external_cursor_observer;
pub mod r#trait;
pub mod registry;

pub use registry::AdapterRegistry;
pub use r#trait::{AdapterContext, AgentAdapter, AttachSessionInput, CreateSessionInput, EventHandler};
