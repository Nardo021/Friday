use std::path::Path;
use std::sync::Weak;
use std::time::Duration;

use tauri::AppHandle;

use crate::commands::intent_commands::SubmitQuickInputResult;
use crate::core::event::{
    AgentEvent, AgentSessionType, FridaySession, FridaySessionStatus, MessageRole, SessionRepo,
    is_running_status, now_iso,
};
use crate::core::intent_router::{
    ControlAction, IntentRouter, QuickIntent, RouteContext, RouteResult,
};
use crate::core::event_bus::emit_agent_event;
use crate::errors::{AppError, AppResult};
use crate::discovery::git_info::{git_branch_with_timeout, git_remote_origin_url, match_project_id, repo_name_from_path};
use crate::storage::{
    EventsRepo, IdeasRepo, MessagesRepo, ProjectsRepo, SessionsRepo,
};

use super::AgentCore;

impl AgentCore {
    pub async fn route_quick_input(
        &self,
        text: String,
        session_id: Option<String>,
        project_id: Option<String>,
        mode: Option<String>,
    ) -> AppResult<RouteResult> {
        let active_session = if let Some(ref sid) = session_id {
            self.session_manager.lock().await.get(sid).ok()
        } else {
            self.session_manager
                .lock()
                .await
                .active_session()
        };

        let has_running = self
            .list_active_sessions()
            .await?
            .into_iter()
            .any(|s| is_running_status(s.status));

        let ctx = RouteContext {
            text,
            session_id,
            project_id,
            mode: mode.unwrap_or_else(|| "local_cli".into()),
            active_session,
            has_running_session: has_running,
        };

        IntentRouter::route(ctx).await
    }

    pub async fn submit_quick_input(
        &self,
        app: AppHandle,
        text: String,
        session_id: Option<String>,
        project_id: Option<String>,
        mode: Option<String>,
    ) -> AppResult<SubmitQuickInputResult> {
        let route = self
            .route_quick_input(text, session_id, project_id, mode)
            .await?;
        self.execute_route(app, route).await
    }

    pub async fn execute_quick_intent(
        &self,
        app: AppHandle,
        intent: QuickIntent,
    ) -> AppResult<SubmitQuickInputResult> {
        let route = RouteResult {
            intent,
            confidence: 1.0,
            source: "manual".into(),
            status_message: None,
        };
        self.execute_route(app, route).await
    }

    async fn execute_route(
        &self,
        app: AppHandle,
        route: RouteResult,
    ) -> AppResult<SubmitQuickInputResult> {
        match route.intent.clone() {
            QuickIntent::Clarify { .. } => Ok(SubmitQuickInputResult {
                route,
                executed: false,
                message: None,
                session_id: None,
            }),
            QuickIntent::OpenChat => {
                let _ = crate::system::window_manager::open_panel(&app);
                Ok(SubmitQuickInputResult {
                    route,
                    executed: true,
                    message: Some("Opened Friday Panel".into()),
                    session_id: None,
                })
            }
            QuickIntent::QueryStatus => {
                let session = self.session_manager.lock().await.active_session();
                let session_id = session.as_ref().map(|s| s.id.clone());
                let message = if let Some(ref s) = session {
                    let recent = MessagesRepo::new(&self.db)
                        .list_for_session(&s.id)?
                        .into_iter()
                        .rev()
                        .take(3)
                        .map(|m| format!("{}: {}", m.role, m.content))
                        .collect::<Vec<_>>();
                    IntentRouter::format_status_summary(&s, &recent)
                } else {
                    "No active session.".into()
                };
                Ok(SubmitQuickInputResult {
                    route,
                    executed: true,
                    message: Some(message),
                    session_id,
                })
            }
            QuickIntent::Control { action, session_id } => {
                let sid = if let Some(sid) = session_id.clone() {
                    Some(sid)
                } else {
                    self.session_manager
                        .lock()
                        .await
                        .active_session()
                        .map(|s| s.id)
                };
                match action {
                    ControlAction::Stop => {
                        if let Some(sid) = sid {
                            self.close_session_safely(app.clone(), &sid).await?;
                            Ok(SubmitQuickInputResult {
                                route,
                                executed: true,
                                message: Some("Session stopped.".into()),
                                session_id: Some(sid),
                            })
                        } else {
                            Ok(SubmitQuickInputResult {
                                route,
                                executed: false,
                                message: Some("No session to stop.".into()),
                                session_id: None,
                            })
                        }
                    }
                    ControlAction::Pause | ControlAction::Resume => Ok(SubmitQuickInputResult {
                        route,
                        executed: false,
                        message: Some("Pause/resume is not supported yet.".into()),
                        session_id: sid,
                    }),
                }
            }
            QuickIntent::SaveIdea {
                title,
                body,
                project_id,
                session_id,
            } => {
                let idea = IdeasRepo::new(&self.db).insert(
                    &title,
                    &body,
                    project_id.as_deref(),
                    session_id.as_deref(),
                )?;
                Ok(SubmitQuickInputResult {
                    route,
                    executed: true,
                    message: Some(format!("Idea saved: {}", idea.title)),
                    session_id: session_id.clone(),
                })
            }
            QuickIntent::FollowUp { session_id, text } => {
                self.follow_up(app, &session_id, text.clone()).await?;
                Ok(SubmitQuickInputResult {
                    route,
                    executed: true,
                    message: None,
                    session_id: Some(session_id.clone()),
                })
            }
            QuickIntent::NewTask {
                project_id,
                mode,
                prompt,
            } => {
                let session_type = match mode.as_str() {
                    "cloud_agent" => AgentSessionType::CursorCloud,
                    "sdk_local" => AgentSessionType::CursorSdkLocal,
                    _ => AgentSessionType::FridayOwnedCli,
                };
                let session = self
                    .create_session(app, session_type, project_id.clone(), prompt.clone())
                    .await?;
                Ok(SubmitQuickInputResult {
                    route,
                    executed: true,
                    message: Some(format!("Started session: {}", session.title)),
                    session_id: Some(session.id),
                })
            }
        }
    }

    pub fn export_session_markdown(&self, session_id: &str) -> AppResult<String> {
        let session = SessionsRepo::new(&self.db).get(session_id)?;
        let events = EventsRepo::new(&self.db).list_for_session(session_id)?;
        let messages = MessagesRepo::new(&self.db).list_for_session(session_id)?;

        let mut md = format!("# {}\n\n", session.title);
        md.push_str(&format!("- Status: {:?}\n", session.status));
        if let Some(prompt) = &session.prompt {
            md.push_str(&format!("- Prompt: {prompt}\n"));
        }
        md.push('\n');
        md.push_str("## Messages\n\n");
        for m in messages {
            md.push_str(&format!("**{}** ({})\n\n{}\n\n", m.role, m.created_at, m.content));
        }
        md.push_str("## Events\n\n");
        for e in events {
            md.push_str(&format!("- `{}`\n", serde_json::to_string(&e)?));
        }
        Ok(md)
    }

    pub fn delete_session(&self, session_id: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM messages WHERE session_id = ?1", [session_id])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            conn.execute(
                "DELETE FROM session_events WHERE session_id = ?1",
                [session_id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            let _ = conn.execute("DELETE FROM events WHERE session_id = ?1", [session_id]);
            conn.execute("DELETE FROM sessions WHERE id = ?1", [session_id])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })?;
        Ok(())
    }

    pub fn enrich_session_repo(
        &self,
        project_id: &str,
        cwd: &str,
    ) -> AppResult<(SessionRepo, Option<String>)> {
        let project = ProjectsRepo::new(&self.db).get(project_id)?;
        let branch = git_branch_with_timeout(Path::new(cwd), Duration::from_secs(2));
        let remote_url = project
            .remote_url
            .clone()
            .or_else(|| git_remote_origin_url(Path::new(cwd)));
        let repo = SessionRepo {
            id: project.id.clone(),
            name: project.name.clone(),
            local_path: Some(cwd.to_string()),
            remote_url,
            branch: branch.clone(),
        };
        Ok((repo, branch))
    }

    pub fn match_external_project(&self, cwd: &str) -> AppResult<Option<(String, SessionRepo)>> {
        let projects = ProjectsRepo::new(&self.db).list()?;
        let pairs: Vec<(String, String)> = projects
            .iter()
            .map(|p| (p.id.clone(), p.path.clone()))
            .collect();
        let cwd_path = Path::new(cwd);
        let Some((project_id, _name)) = match_project_id(cwd_path, &pairs) else {
            let repo = SessionRepo {
                id: repo_name_from_path(cwd_path),
                name: repo_name_from_path(cwd_path),
                local_path: Some(cwd.to_string()),
                remote_url: None,
                branch: git_branch_with_timeout(cwd_path, Duration::from_secs(2)),
            };
            return Ok(Some((String::new(), repo)));
        };
        let project = projects.iter().find(|p| p.id == project_id).unwrap();
        let branch = git_branch_with_timeout(cwd_path, Duration::from_secs(2));
        let remote_url = git_remote_origin_url(cwd_path);
        let repo = SessionRepo {
            id: project.id.clone(),
            name: project.name.clone(),
            local_path: Some(cwd.to_string()),
            remote_url,
            branch,
        };
        Ok(Some((project_id, repo)))
    }
}
