use std::sync::{Arc, RwLock, Weak};

use tauri::AppHandle;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::adapters::{
    AdapterContext, AdapterRegistry, CreateSessionInput, EventHandler,
};
use crate::adapters::registry::{
    ADAPTER_CURSOR_CLI_LOCAL, ADAPTER_CURSOR_CLOUD_AGENT,
};
use crate::core::event::{
    AgentEvent, AgentSessionType, FridaySession, FridaySessionStatus, SessionOwnership,
    SessionRepo, is_running_status, now_iso,
};
use crate::core::event_bus::emit_agent_event;
use crate::core::session_manager::SessionManager;
use crate::errors::{AppError, AppResult};
use crate::process::ProcessSupervisor;
use crate::pty::PtyManager;
use crate::security::{ApprovalManager, ProjectAllowlist, SecretStore};
use crate::storage::{
    CloudAgentsRepo, Database, ProjectsRepo, QueuedInstructionsRepo, SessionsRepo,
    SettingsRepo,
};
use crate::storage::settings_repo::{CloudSettings, CursorSettings, FridaySettings};

pub struct AgentCore {
    pub session_manager: Mutex<SessionManager>,
    pub process_supervisor: Arc<ProcessSupervisor>,
    pub pty_manager: Arc<PtyManager>,
    pub approval_manager: Mutex<ApprovalManager>,
    pub adapter_registry: AdapterRegistry,
    pub db: Arc<Database>,
    pub(crate) app_handle: Mutex<Option<AppHandle>>,
    pub(crate) self_weak: std::sync::Mutex<Option<Weak<AgentCore>>>,
    settings: RwLock<FridaySettings>,
    event_handler: RwLock<Option<EventHandler>>,
}

impl AgentCore {
    pub fn new(process_supervisor: Arc<ProcessSupervisor>, db: Arc<Database>) -> Self {
        let settings = SettingsRepo::new(&db)
            .get()
            .unwrap_or_default();
        Self {
            session_manager: Mutex::new(SessionManager::new()),
            process_supervisor,
            pty_manager: Arc::new(PtyManager::new()),
            approval_manager: Mutex::new(ApprovalManager::new()),
            adapter_registry: AdapterRegistry::new(),
            db,
            app_handle: Mutex::new(None),
            self_weak: std::sync::Mutex::new(None),
            settings: RwLock::new(settings),
            event_handler: RwLock::new(None),
        }
    }

    pub fn bind_weak(self: &Arc<Self>, weak: Weak<AgentCore>) {
        if let Ok(mut slot) = self.self_weak.lock() {
            *slot = Some(weak);
        }
    }

    pub async fn init(&self, app: AppHandle) -> AppResult<()> {
        *self.app_handle.lock().await = Some(app.clone());
        self.reload_settings_cache()?;
        self.install_event_handler(app.clone());

        let sessions = SessionsRepo::new(&self.db).list()?;
        self.session_manager.lock().await.load_from_db(sessions);

        self.resume_cloud_sessions(app).await?;

        Ok(())
    }

    pub fn settings_snapshot(&self) -> FridaySettings {
        self.settings.read().unwrap_or_else(|e| e.into_inner()).clone()
    }

    pub fn reload_settings_cache(&self) -> AppResult<()> {
        let settings = SettingsRepo::new(&self.db).get()?;
        *self.settings.write().unwrap_or_else(|e| e.into_inner()) = settings;
        Ok(())
    }

    fn install_event_handler(&self, app: AppHandle) {
        let self_weak = self
            .self_weak
            .lock()
            .ok()
            .and_then(|g| g.clone())
            .expect("AgentCore weak ref");
        let tx = crate::core::event_ingest::spawn_event_worker(
            self_weak,
            app.clone(),
            self.db.clone(),
        );
        let handler = crate::core::event_ingest::make_event_handler(tx, app);
        *self.event_handler.write().unwrap_or_else(|e| e.into_inner()) = Some(handler);
    }

    async fn resume_cloud_sessions(&self, _app: AppHandle) -> AppResult<()> {
        let records = CloudAgentsRepo::new(&self.db).list_with_run()?;
        let settings = self.settings_snapshot();
        let ctx = self.build_context(settings.cursor.clone(), settings.cloud.clone());

        for record in records {
            let session = match SessionsRepo::new(&self.db).get(&record.session_id) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if session.session_type != AgentSessionType::CursorCloud {
                continue;
            }
            if !is_running_status(session.status) {
                continue;
            }
            let Some(run_id) = record.run_id else {
                continue;
            };
            crate::adapters::cursor_cloud_agent::spawn_run_stream(
                record.session_id,
                record.agent_id,
                run_id,
                ctx.event_handler.clone(),
                self.db.clone(),
            );
        }
        Ok(())
    }

    pub async fn create_session(
        &self,
        app: AppHandle,
        session_type: AgentSessionType,
        project_id: String,
        prompt: String,
    ) -> AppResult<FridaySession> {
        self.session_manager
            .lock()
            .await
            .ensure_can_create(session_type)?;

        let (cwd, adapter_id) = match session_type {
            AgentSessionType::FridayOwnedCli => {
                let projects_repo = ProjectsRepo::new(&self.db);
                let project = projects_repo.get(&project_id)?;
                ProjectAllowlist::validate_path(&projects_repo, &project.path)?;
                if !ProjectAllowlist::is_trusted(&project) {
                    return Err(AppError::ProjectNotAllowed(
                        "Project is not trusted".into(),
                    ));
                }
                projects_repo.touch(&project_id)?;
                let _ = projects_repo.refresh_remote_url_from_git(&project_id);
                let adapter_id = if project.default_adapter_id.is_empty() {
                    ADAPTER_CURSOR_CLI_LOCAL.to_string()
                } else {
                    project.default_adapter_id.clone()
                };
                (project.path, adapter_id)
            }
            AgentSessionType::CursorCloud => {
                if !SecretStore::has_cursor_api_key()? {
                    return Err(AppError::Other(
                        "Cursor API key not configured. Add it in Settings.".into(),
                    ));
                }
                let projects_repo = ProjectsRepo::new(&self.db);
                let project = projects_repo.get(&project_id)?;
                ProjectAllowlist::validate_path(&projects_repo, &project.path)?;
                if !ProjectAllowlist::is_trusted(&project) {
                    return Err(AppError::ProjectNotAllowed(
                        "Project is not trusted".into(),
                    ));
                }
                projects_repo.touch(&project_id)?;
                let _ = projects_repo.refresh_remote_url_from_git(&project_id);
                (project.path.clone(), ADAPTER_CURSOR_CLOUD_AGENT.to_string())
            }
            _ => {
                return Err(AppError::Other(format!(
                    "create_session does not support {session_type:?}"
                )));
            }
        };

        let session_id = Uuid::new_v4().to_string();
        let settings = self.settings_snapshot();
        let ctx = self.build_context(settings.cursor.clone(), settings.cloud.clone());

        let adapter = self.adapter_registry.get_adapter(&adapter_id)?;
        let mut session = adapter
            .create_session(
                CreateSessionInput {
                    session_id: session_id.clone(),
                    session_type,
                    project_id: project_id.clone(),
                    prompt: prompt.clone(),
                    cwd: cwd.clone(),
                    model: None,
                },
                &ctx,
            )
            .await?;

        if let Ok((repo, _branch)) = self.enrich_session_repo(&project_id, &cwd) {
            session.repo = Some(repo);
        }

        {
            let mut mgr = self.session_manager.lock().await;
            mgr.create(session.clone());
        }

        SessionsRepo::new(&self.db).insert(&session)?;
        Ok(session)
    }

    pub async fn close_session_safely(
        &self,
        app: AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let session = self.session_manager.lock().await.get(session_id)?;

        match session.ownership {
            SessionOwnership::External => {
                self.session_manager
                    .lock()
                    .await
                    .update_status(session_id, FridaySessionStatus::Stopped)?;
                SessionsRepo::new(&self.db)
                    .update_status(session_id, FridaySessionStatus::Stopped, None)?;
            }
            SessionOwnership::Friday => {
                let settings = self.settings_snapshot();
                let ctx = self.build_context(settings.cursor.clone(), settings.cloud.clone());
                let adapter = self.adapter_registry.get_adapter(&session.adapter_id)?;
                adapter.stop_session(session_id, &ctx).await?;
                self.session_manager
                    .lock()
                    .await
                    .update_status(session_id, FridaySessionStatus::Stopped)?;
                SessionsRepo::new(&self.db)
                    .update_status(session_id, FridaySessionStatus::Stopped, None)?;
            }
        }

        let event = AgentEvent::AgentStatus {
            session_id: session_id.to_string(),
            status: FridaySessionStatus::Stopped,
            message: Some("Session closed".into()),
            timestamp: now_iso(),
        };
        emit_agent_event(&app, &event)?;
        Ok(())
    }

    pub async fn select_active_session(&self, session_id: &str) -> AppResult<()> {
        self.session_manager.lock().await.set_active(session_id)
    }

    pub async fn resize_terminal(
        &self,
        session_id: &str,
        cols: u16,
        rows: u16,
    ) -> AppResult<()> {
        if let Some(pty_id) = self.pty_manager.pty_id_for_session(session_id) {
            self.pty_manager.resize(&pty_id, cols, rows)?;
        }
        Ok(())
    }

    pub async fn follow_up(
        &self,
        app: AppHandle,
        session_id: &str,
        message: String,
    ) -> AppResult<()> {
        let session = self.session_manager.lock().await.get(session_id)?;
        let settings = self.settings_snapshot();
        let use_pty = settings.cursor.use_pty;
        let ctx = self.build_context(settings.cursor.clone(), settings.cloud.clone());
        let adapter = self.adapter_registry.get_adapter(&session.adapter_id)?;
        match adapter.send_message(session_id, &message, &ctx).await {
            Ok(()) => {}
            Err(e) if !use_pty => {
                QueuedInstructionsRepo::new(&self.db).enqueue(session_id, &message)?;
                let event = AgentEvent::AgentMessage {
                    session_id: session_id.to_string(),
                    role: crate::core::event::MessageRole::System,
                    content: format!("Instruction queued (pipe mode): {message}"),
                    timestamp: now_iso(),
                };
                emit_agent_event(&app, &event)?;
                return Ok(());
            }
            Err(e) => return Err(e),
        }

        (ctx.event_handler)(AgentEvent::AgentMessage {
            session_id: session_id.to_string(),
            role: crate::core::event::MessageRole::User,
            content: message,
            timestamp: now_iso(),
        });

        Ok(())
    }

    pub async fn list_active_sessions(&self) -> AppResult<Vec<FridaySession>> {
        Ok(self
            .session_manager
            .lock()
            .await
            .list()
            .into_iter()
            .filter(|s| {
                is_running_status(s.status) || s.status == FridaySessionStatus::Discovered
            })
            .collect())
    }

    // Backward-compatible aliases
    pub async fn start_session(
        &self,
        app: AppHandle,
        project_id: String,
        prompt: String,
    ) -> AppResult<FridaySession> {
        self.create_session(app, AgentSessionType::FridayOwnedCli, project_id, prompt)
            .await
    }

    pub async fn stop_session(&self, app: AppHandle, session_id: &str) -> AppResult<()> {
        self.close_session_safely(app, session_id).await
    }

    pub async fn send_message(
        &self,
        app: AppHandle,
        session_id: &str,
        message: String,
    ) -> AppResult<()> {
        self.follow_up(app, session_id, message).await
    }

    pub async fn approve_command(&self, app: AppHandle, approval_id: &str) -> AppResult<()> {
        let session_id = {
            let mgr = self.approval_manager.lock().await;
            mgr.pending_session_id(approval_id)
        };
        self.approval_manager.lock().await.approve(approval_id)?;
        if let Some(sid) = session_id {
            self.session_manager
                .lock()
                .await
                .update_status(&sid, FridaySessionStatus::RunningCommand)?;
            SessionsRepo::new(&self.db)
                .update_status(&sid, FridaySessionStatus::RunningCommand, None)?;
            let event = AgentEvent::AgentStatus {
                session_id: sid,
                status: FridaySessionStatus::RunningCommand,
                message: Some("Approval granted".into()),
                timestamp: now_iso(),
            };
            emit_agent_event(&app, &event)?;
        }
        Ok(())
    }

    pub async fn reject_command(&self, app: AppHandle, approval_id: &str) -> AppResult<()> {
        let session_id = {
            let mgr = self.approval_manager.lock().await;
            mgr.pending_session_id(approval_id)
        };
        self.approval_manager.lock().await.reject(approval_id)?;
        if let Some(sid) = session_id {
            self.close_session_safely(app, &sid).await?;
        } else {
            let event = AgentEvent::AgentStatus {
                session_id: String::new(),
                status: FridaySessionStatus::Stopped,
                message: Some("Approval rejected".into()),
                timestamp: now_iso(),
            };
            emit_agent_event(&app, &event)?;
        }
        Ok(())
    }

    pub async fn try_flush_instruction_queue(
        &self,
        app: &AppHandle,
        session_id: &str,
    ) -> AppResult<()> {
        let settings = self.settings_snapshot();
        if settings.cursor.use_pty {
            return Ok(());
        }
        let next = QueuedInstructionsRepo::new(&self.db).pop_next(session_id)?;
        if let Some(item) = next {
            self.follow_up(app.clone(), session_id, item.text).await?;
        }
        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> AppResult<FridaySession> {
        self.session_manager.lock().await.get(session_id)
    }

    pub fn list_sessions(&self) -> AppResult<Vec<FridaySession>> {
        SessionsRepo::new(&self.db).list()
    }

    pub async fn dispatch_event(&self, app: AppHandle, event: AgentEvent) -> AppResult<()> {
        crate::core::event_dispatch::dispatch_event(self, app, self.db.clone(), event).await
    }

    pub(crate) fn build_context(
        &self,
        cursor_settings: CursorSettings,
        cloud_settings: CloudSettings,
    ) -> AdapterContext {
        let handler = self
            .event_handler
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .expect("event handler not installed");

        AdapterContext {
            supervisor: self.process_supervisor.clone(),
            pty_manager: self.pty_manager.clone(),
            event_handler: handler,
            cursor_settings,
            cloud_settings,
            db: self.db.clone(),
        }
    }
}
