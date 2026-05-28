use std::sync::Arc;

use tauri::AppHandle;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::adapters::adapter_trait::StartSessionInput;
use crate::adapters::{start_cursor_session, AdapterRegistry};
use crate::core::event::{AgentEvent, AgentSession, AgentStatus, now_iso};
use crate::core::event_bus::emit_agent_event;
use crate::core::session_manager::SessionManager;
use crate::errors::{AppError, AppResult};
use crate::process::ProcessSupervisor;
use crate::security::{ApprovalManager, CommandPolicy, ProjectAllowlist, SecretRedactor};
use crate::storage::{
    Database, EventsRepo, MessagesRepo, ProjectsRepo, SessionsRepo, SettingsRepo, logs_dir,
};

pub struct AgentCore {
    pub session_manager: Mutex<SessionManager>,
    pub process_supervisor: Arc<ProcessSupervisor>,
    pub approval_manager: Mutex<ApprovalManager>,
    pub adapter_registry: AdapterRegistry,
    pub db: Arc<Database>,
}

impl AgentCore {
    pub fn new(process_supervisor: Arc<ProcessSupervisor>, db: Arc<Database>) -> Self {
        Self {
            session_manager: Mutex::new(SessionManager::new()),
            process_supervisor,
            approval_manager: Mutex::new(ApprovalManager::new()),
            adapter_registry: AdapterRegistry::new(),
            db,
        }
    }

    pub async fn init(&self) -> AppResult<()> {
        let sessions = SessionsRepo::new(&self.db).list()?;
        self.session_manager.lock().await.load_from_db(sessions);
        Ok(())
    }

    pub async fn start_session(
        &self,
        app: AppHandle,
        project_id: String,
        prompt: String,
    ) -> AppResult<AgentSession> {
        if self.session_manager.lock().await.has_active_running() {
            return Err(AppError::SessionAlreadyRunning);
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

        let session_id = Uuid::new_v4().to_string();
        let title = truncate_title(&prompt);
        let settings = SettingsRepo::new(&self.db).get()?;

        let session = {
            let mut mgr = self.session_manager.lock().await;
            mgr.create_session(
                session_id.clone(),
                title,
                project.default_adapter_id.clone(),
                project_id,
                project.path.clone(),
                prompt.clone(),
            )
        };

        SessionsRepo::new(&self.db).insert(&session)?;

        let app_clone = app.clone();
        let db = self.db.clone();

        let handler: Arc<dyn Fn(AgentEvent) + Send + Sync> = Arc::new(move |event: AgentEvent| {
            let app = app_clone.clone();
            let db = db.clone();
            tauri::async_runtime::spawn(async move {
                let _ = handle_event(app, db, event).await;
            });
        });

        let pid = start_cursor_session(
            self.process_supervisor.clone(),
            StartSessionInput {
                session_id: session_id.clone(),
                prompt,
                cwd: project.path.clone(),
                model: None,
            },
            settings.cursor,
            handler,
        )
        .await?;

        {
            let mut mgr = self.session_manager.lock().await;
            mgr.set_pid(&session_id, pid)?;
        }

        self.session_manager.lock().await.get(&session_id)
    }

    pub async fn stop_session(&self, app: AppHandle, session_id: &str) -> AppResult<()> {
        self.process_supervisor.stop_session(session_id, 3).await?;

        {
            let mut mgr = self.session_manager.lock().await;
            mgr.set_status(session_id, AgentStatus::Cancelled)?;
        }

        SessionsRepo::new(&self.db)
            .update_status(session_id, AgentStatus::Cancelled, None)?;

        let event = AgentEvent::AgentStatus {
            session_id: session_id.to_string(),
            status: AgentStatus::Cancelled,
            message: Some("Session stopped by user".into()),
            timestamp: now_iso(),
        };

        emit_agent_event(&app, &event)?;
        Ok(())
    }

    pub async fn send_message(
        &self,
        app: AppHandle,
        session_id: &str,
        message: String,
    ) -> AppResult<()> {
        if self.session_manager.lock().await.has_active_running() {
            return Err(AppError::Other(
                "Cannot send message while session is running. Queue support coming in v0.2.".into(),
            ));
        }

        let project_id = self
            .session_manager
            .lock()
            .await
            .get(session_id)?
            .project_id;
        self.start_session(app, project_id, message).await?;
        Ok(())
    }

    pub async fn approve_command(&self, app: AppHandle, approval_id: &str) -> AppResult<()> {
        self.approval_manager.lock().await.approve(approval_id)?;

        let event = AgentEvent::AgentStatus {
            session_id: String::new(),
            status: AgentStatus::RunningCommand,
            message: Some("Approval granted".into()),
            timestamp: now_iso(),
        };
        emit_agent_event(&app, &event)?;
        Ok(())
    }

    pub async fn reject_command(&self, app: AppHandle, approval_id: &str) -> AppResult<()> {
        self.approval_manager.lock().await.reject(approval_id)?;

        let event = AgentEvent::AgentStatus {
            session_id: String::new(),
            status: AgentStatus::Cancelled,
            message: Some("Approval rejected".into()),
            timestamp: now_iso(),
        };
        emit_agent_event(&app, &event)?;
        Ok(())
    }

    pub async fn get_session(&self, session_id: &str) -> AppResult<AgentSession> {
        self.session_manager.lock().await.get(session_id)
    }

    pub fn list_sessions(&self) -> AppResult<Vec<AgentSession>> {
        SessionsRepo::new(&self.db).list()
    }
}

async fn handle_event(app: AppHandle, db: Arc<Database>, event: AgentEvent) -> AppResult<()> {
    let session_id = event.session_id().to_string();

    if let AgentEvent::CommandStarted { command, .. } = &event {
        let settings = SettingsRepo::new(&db).get()?;
        if CommandPolicy::requires_approval(command, &settings.security) {
            // High-risk commands surface approval.required via mapper; policy gate here for logging
        }
    }

    EventsRepo::new(&db).insert(&event)?;
    MessagesRepo::new(&db).insert_from_event(&event)?;
    append_session_log(&session_id, &event)?;

    if let AgentEvent::AgentStatus { status, .. } = &event {
        SessionsRepo::new(&db).update_status(&session_id, *status, None)?;
    }

    if let AgentEvent::SessionCompleted { summary, .. } = &event {
        SessionsRepo::new(&db).update_status(
            &session_id,
            AgentStatus::Completed,
            summary.as_deref(),
        )?;
    }

    if let AgentEvent::SessionError { .. } = &event {
        SessionsRepo::new(&db).update_status(&session_id, AgentStatus::Error, None)?;
    }

    emit_agent_event(&app, &event)?;
    Ok(())
}

fn truncate_title(prompt: &str) -> String {
    let trimmed = prompt.trim();
    if trimmed.len() <= 60 {
        trimmed.to_string()
    } else {
        format!("{}...", &trimmed[..57])
    }
}

fn append_session_log(session_id: &str, event: &AgentEvent) -> AppResult<()> {
    let dir = logs_dir()?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{session_id}.log"));
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    let line = SecretRedactor::redact(&serde_json::to_string(event)?);
    use std::io::Write;
    writeln!(file, "{line}")?;
    Ok(())
}
