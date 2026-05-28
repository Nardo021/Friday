pub mod agent_core;
pub mod agent_intent;
pub mod event;
pub mod event_bus;
pub mod event_dispatch;
pub mod event_ingest;
pub mod event_pipeline;
pub mod intent_router;
pub mod session_manager;
pub mod state_machine;

pub use agent_core::AgentCore;
pub use crate::discovery::external_loop::start_discovery_loop;
pub use event::*;
pub use event_bus::{AGENT_EVENT_CHANNEL, emit_agent_event};
pub use session_manager::SessionManager;
