use std::path::PathBuf;
use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::process::process_registry::{
    ManagedProcess, ProcessRegistry, ProcessStatus, SharedProcessRegistry,
};

pub struct SpawnConfig {
    pub session_id: String,
    pub adapter_id: String,
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
}

pub struct ProcessSupervisor {
    registry: SharedProcessRegistry,
    children: tokio::sync::Mutex<std::collections::HashMap<String, Child>>,
}

impl ProcessSupervisor {
    pub fn new(registry: SharedProcessRegistry) -> Self {
        Self {
            registry,
            children: tokio::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    pub async fn spawn_with_output(
        &self,
        config: SpawnConfig,
    ) -> AppResult<(u32, mpsc::UnboundedReceiver<(String, bool)>)> {
        let process_id = Uuid::new_v4().to_string();
        let command_display = format!("{} {}", config.executable, config.args.join(" "));

        let mut cmd = Command::new(&config.executable);
        cmd.args(&config.args)
            .current_dir(&config.cwd)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| AppError::Process(format!("failed to spawn {}: {e}", config.executable)))?;

        let pid = child.id().ok_or_else(|| AppError::Process("no pid".into()))?;

        {
            let mut registry = self.registry.write().await;
            registry.register(ManagedProcess {
                process_id: process_id.clone(),
                session_id: config.session_id.clone(),
                adapter_id: config.adapter_id.clone(),
                pid: Some(pid),
                cwd: config.cwd.clone(),
                command: command_display,
                started_at: chrono::Utc::now(),
                status: ProcessStatus::Running,
            });
        }

        let (tx, rx) = mpsc::unbounded_channel();

        if let Some(stdout) = child.stdout.take() {
            let tx = tx.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send((line, false));
                }
            });
        }

        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = tx.send((line, true));
                }
            });
        }

        self.children
            .lock()
            .await
            .insert(config.session_id.clone(), child);

        Ok((pid, rx))
    }

    pub async fn wait_session(
        &self,
        session_id: String,
        event_handler: std::sync::Arc<dyn Fn(crate::core::event::AgentEvent) + Send + Sync>,
    ) {
        let child = self.children.lock().await.remove(&session_id);
        if let Some(mut child) = child {
            let exit = child.wait().await;
            let status = match exit {
                Ok(s) if s.success() => crate::core::event::FridaySessionStatus::Done,
                Ok(_) => crate::core::event::FridaySessionStatus::Stopped,
                Err(_) => crate::core::event::FridaySessionStatus::Error,
            };
            event_handler(crate::core::event::AgentEvent::AgentStatus {
                session_id: session_id.clone(),
                status,
                message: Some("Process exited".into()),
                timestamp: crate::core::event::now_iso(),
            });
            {
                let mut registry = self.registry.write().await;
                if let Some(proc) = registry.get_mut_by_session(&session_id) {
                    proc.status = crate::process::process_registry::ProcessStatus::Exited;
                }
                registry.remove_by_session(&session_id);
            }
        }
    }

    pub async fn stop_session(&self, session_id: &str, force_after_secs: u64) -> AppResult<()> {
        let mut child = self.children.lock().await.remove(session_id);

        if let Some(ref mut c) = child {
            let _ = c.start_kill();
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(force_after_secs)).await;

        if let Some(mut c) = child {
            let _ = c.kill().await;
            let _ = c.wait().await;
        }

        {
            let mut registry = self.registry.write().await;
            if let Some(proc) = registry.get_mut_by_session(session_id) {
                proc.status = ProcessStatus::Killed;
            }
            registry.remove_by_session(session_id);
        }

        Ok(())
    }
}

pub fn create_registry() -> SharedProcessRegistry {
    std::sync::Arc::new(tokio::sync::RwLock::new(ProcessRegistry::default()))
}
