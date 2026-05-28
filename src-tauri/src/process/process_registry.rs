use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Exited,
    Crashed,
    Killed,
}

#[derive(Debug, Clone)]
pub struct ManagedProcess {
    pub process_id: String,
    pub session_id: String,
    pub adapter_id: String,
    pub pid: Option<u32>,
    pub cwd: PathBuf,
    pub command: String,
    pub started_at: DateTime<Utc>,
    pub status: ProcessStatus,
}

#[derive(Default)]
pub struct ProcessRegistry {
    processes: HashMap<String, ManagedProcess>,
}

impl ProcessRegistry {
    pub fn register(&mut self, process: ManagedProcess) {
        self.processes.insert(process.process_id.clone(), process);
    }

    pub fn get_by_session(&self, session_id: &str) -> Option<&ManagedProcess> {
        self.processes
            .values()
            .find(|p| p.session_id == session_id)
    }

    pub fn get_mut_by_session(&mut self, session_id: &str) -> Option<&mut ManagedProcess> {
        self.processes
            .values_mut()
            .find(|p| p.session_id == session_id)
    }

    pub fn remove_by_session(&mut self, session_id: &str) -> Option<ManagedProcess> {
        let id = self
            .processes
            .values()
            .find(|p| p.session_id == session_id)
            .map(|p| p.process_id.clone());
        id.and_then(|pid| self.processes.remove(&pid))
    }

    pub fn all_pids(&self) -> Vec<u32> {
        self.processes
            .values()
            .filter_map(|p| p.pid)
            .collect()
    }
}

pub type SharedProcessRegistry = Arc<RwLock<ProcessRegistry>>;
