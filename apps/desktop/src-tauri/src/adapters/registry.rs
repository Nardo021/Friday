use std::collections::HashMap;
use std::sync::Arc;

use crate::adapters::cursor_cli_local::CursorCliLocalAdapter;
use crate::adapters::cursor_cloud_agent::CursorCloudAgentAdapter;
use crate::adapters::cursor_sdk_local::CursorSdkLocalAdapter;
use crate::adapters::external_cursor_observer::ExternalCursorObserverAdapter;
use crate::adapters::r#trait::AgentAdapter;
use crate::core::event::{AgentCapabilities, AgentSessionType, AdapterInfo, ControlLevel};
use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;

pub const ADAPTER_CURSOR_CLI_LOCAL: &str = "cursor-cli-local";
pub const ADAPTER_EXTERNAL_CURSOR_OBSERVER: &str = "external-cursor-observer";
pub const ADAPTER_CURSOR_SDK_LOCAL: &str = "cursor-sdk-local";
pub const ADAPTER_CURSOR_CLOUD_AGENT: &str = "cursor-cloud-agent";

pub struct AdapterRegistry {
    adapters: HashMap<String, Arc<dyn AgentAdapter>>,
    infos: HashMap<String, AdapterInfo>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        let mut adapters: HashMap<String, Arc<dyn AgentAdapter>> = HashMap::new();
        let mut infos: HashMap<String, AdapterInfo> = HashMap::new();

        let cursor_cli = Arc::new(CursorCliLocalAdapter);
        adapters.insert(ADAPTER_CURSOR_CLI_LOCAL.into(), cursor_cli);
        infos.insert(
            ADAPTER_CURSOR_CLI_LOCAL.into(),
            AdapterInfo {
                id: ADAPTER_CURSOR_CLI_LOCAL.into(),
                name: "Cursor CLI (Friday)".into(),
                available: true,
                session_type: AgentSessionType::FridayOwnedCli,
                capabilities: friday_owned_cli_capabilities(),
            },
        );

        let external = Arc::new(ExternalCursorObserverAdapter);
        adapters.insert(ADAPTER_EXTERNAL_CURSOR_OBSERVER.into(), external);
        infos.insert(
            ADAPTER_EXTERNAL_CURSOR_OBSERVER.into(),
            AdapterInfo {
                id: ADAPTER_EXTERNAL_CURSOR_OBSERVER.into(),
                name: "External Cursor Observer".into(),
                available: true,
                session_type: AgentSessionType::ExternalCli,
                capabilities: external_cursor_observer_capabilities(),
            },
        );

        let sdk = Arc::new(CursorSdkLocalAdapter);
        adapters.insert(ADAPTER_CURSOR_SDK_LOCAL.into(), sdk);
        infos.insert(
            ADAPTER_CURSOR_SDK_LOCAL.into(),
            AdapterInfo {
                id: ADAPTER_CURSOR_SDK_LOCAL.into(),
                name: "Cursor SDK (Local)".into(),
                available: false,
                session_type: AgentSessionType::CursorSdkLocal,
                capabilities: stub_capabilities(),
            },
        );

        let cloud = Arc::new(CursorCloudAgentAdapter);
        adapters.insert(ADAPTER_CURSOR_CLOUD_AGENT.into(), cloud);
        infos.insert(
            ADAPTER_CURSOR_CLOUD_AGENT.into(),
            AdapterInfo {
                id: ADAPTER_CURSOR_CLOUD_AGENT.into(),
                name: "Cursor Cloud Agent".into(),
                available: false,
                session_type: AgentSessionType::CursorCloud,
                capabilities: cloud_capabilities(),
            },
        );

        Self { adapters, infos }
    }

    pub fn list(&self) -> Vec<AdapterInfo> {
        let mut items: Vec<AdapterInfo> = self.infos.values().cloned().collect();
        let cloud_available = SecretStore::has_cursor_api_key().unwrap_or(false);
        for info in &mut items {
            if info.id == ADAPTER_CURSOR_CLOUD_AGENT {
                info.available = cloud_available;
            }
        }
        items
    }

    pub fn get_info(&self, id: &str) -> AppResult<AdapterInfo> {
        let mut info = self
            .infos
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::AdapterNotFound(id.to_string()))?;
        if id == ADAPTER_CURSOR_CLOUD_AGENT {
            info.available = SecretStore::has_cursor_api_key().unwrap_or(false);
        }
        Ok(info)
    }

    pub fn get_adapter(&self, id: &str) -> AppResult<Arc<dyn AgentAdapter>> {
        self.adapters
            .get(id)
            .cloned()
            .ok_or_else(|| AppError::AdapterNotFound(id.to_string()))
    }

    pub fn default_adapter_for_type(&self, session_type: AgentSessionType) -> AppResult<String> {
        let id = match session_type {
            AgentSessionType::FridayOwnedCli => ADAPTER_CURSOR_CLI_LOCAL,
            AgentSessionType::ExternalCli => ADAPTER_EXTERNAL_CURSOR_OBSERVER,
            AgentSessionType::CursorSdkLocal => ADAPTER_CURSOR_SDK_LOCAL,
            AgentSessionType::CursorCloud => ADAPTER_CURSOR_CLOUD_AGENT,
        };
        Ok(id.to_string())
    }
}

fn friday_owned_cli_capabilities() -> AgentCapabilities {
    AgentCapabilities {
        can_create: true,
        can_attach: false,
        can_observe: true,
        can_send_follow_up: true,
        can_stop: true,
        can_resume: false,
        can_stream_events: true,
        can_read_artifacts: false,
        can_open_pr: false,
        control_level: ControlLevel::Full,
    }
}

fn external_cursor_observer_capabilities() -> AgentCapabilities {
    AgentCapabilities {
        can_create: false,
        can_attach: true,
        can_observe: true,
        can_send_follow_up: false,
        can_stop: false,
        can_resume: false,
        can_stream_events: false,
        can_read_artifacts: false,
        can_open_pr: false,
        control_level: ControlLevel::Observe,
    }
}

fn cloud_capabilities() -> AgentCapabilities {
    AgentCapabilities {
        can_create: true,
        can_attach: false,
        can_observe: true,
        can_send_follow_up: true,
        can_stop: true,
        can_resume: true,
        can_stream_events: true,
        can_read_artifacts: true,
        can_open_pr: true,
        control_level: ControlLevel::Full,
    }
}

fn stub_capabilities() -> AgentCapabilities {
    AgentCapabilities {
        can_create: false,
        can_attach: false,
        can_observe: false,
        can_send_follow_up: false,
        can_stop: false,
        can_resume: false,
        can_stream_events: false,
        can_read_artifacts: false,
        can_open_pr: false,
        control_level: ControlLevel::None,
    }
}
