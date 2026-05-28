pub mod client;
pub mod mapper;

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::StreamExt;

use crate::adapters::cursor_cloud_agent::client::{
    CreateAgentRequest, CursorCloudClient, PromptBody, RepoConfig,
};
use crate::adapters::cursor_cloud_agent::mapper::{
    artifact_events, map_run_status, map_run_terminal, map_sse_event,
};
use crate::adapters::r#trait::{
    AdapterContext, AgentAdapter, AttachSessionInput, CreateSessionInput, EventHandler,
};
use crate::adapters::registry::ADAPTER_CURSOR_CLOUD_AGENT;
use crate::core::event::{
    AgentEvent, AgentSessionType, ControlLevel, FridaySession, FridaySessionStatus,
    MessageRole, SessionCloud, SessionOwnership, now_iso,
};
use crate::discovery::git_info::git_remote_origin_url;
use crate::errors::{AppError, AppResult};
use crate::storage::cloud_agents_repo::CloudAgentsRepo;

pub struct CursorCloudAgentAdapter;

pub fn spawn_run_stream(
    session_id: String,
    agent_id: String,
    run_id: String,
    handler: EventHandler,
    db: Arc<crate::storage::Database>,
) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = stream_run_loop(&session_id, &agent_id, &run_id, &handler, &db).await {
            handler(AgentEvent::SessionError {
                session_id: session_id.clone(),
                error: e.to_string(),
                timestamp: now_iso(),
            });
        }
    });
}

async fn stream_run_loop(
    session_id: &str,
    agent_id: &str,
    run_id: &str,
    handler: &EventHandler,
    db: &Arc<crate::storage::Database>,
) -> AppResult<()> {
    let client = CursorCloudClient::new()?;
    let mut last_event_id: Option<String> = None;

    loop {
        let resp = match client
            .stream_run(agent_id, run_id, last_event_id.as_deref())
            .await
        {
            Ok(r) => r,
            Err(e) => {
                if e.to_string().contains("410") {
                    break;
                }
                return poll_terminal_run(session_id, agent_id, run_id, handler, &client, db).await;
            }
        };

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut current_event = String::new();
        let mut current_data = String::new();
        let mut stream_done = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| AppError::Other(format!("stream read failed: {e}")))?;
            buffer.push_str(&String::from_utf8_lossy(&chunk));

            while let Some(pos) = buffer.find("\n\n") {
                let block = buffer[..pos].to_string();
                buffer = buffer[pos + 2..].to_string();

                current_event.clear();
                current_data.clear();

                for line in block.lines() {
                    if let Some(id) = line.strip_prefix("id: ") {
                        last_event_id = Some(id.trim().to_string());
                    } else if let Some(ev) = line.strip_prefix("event: ") {
                        current_event = ev.trim().to_string();
                    } else if let Some(data) = line.strip_prefix("data: ") {
                        current_data = data.trim().to_string();
                    }
                }

                if current_event.is_empty() || current_data.is_empty() {
                    continue;
                }

                for event in map_sse_event(session_id, &current_event, &current_data) {
                    handler(event);
                }

                if current_event == "done" {
                    stream_done = true;
                    break;
                }
            }

            if stream_done {
                break;
            }
        }

        if stream_done {
            break;
        }
    }

    poll_terminal_run(session_id, agent_id, run_id, handler, &client, db).await
}

async fn poll_terminal_run(
    session_id: &str,
    agent_id: &str,
    run_id: &str,
    handler: &EventHandler,
    client: &CursorCloudClient,
    db: &Arc<crate::storage::Database>,
) -> AppResult<()> {
    let run = client.get_run(agent_id, run_id).await?;
    let status = run.status.to_uppercase();

    CloudAgentsRepo::new(db).update_run(session_id, run_id, Some(&status))?;

    if matches!(status.as_str(), "CREATING" | "RUNNING") {
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        return Box::pin(stream_run_loop(session_id, agent_id, run_id, handler, db)).await;
    }

    for event in map_run_terminal(session_id, &run) {
        handler(event);
    }

    if status == "FINISHED" {
        if let Ok(artifacts) = client.list_artifacts(agent_id).await {
            for event in artifact_events(session_id, &artifacts) {
                handler(event);
            }
        }
    }

    Ok(())
}

#[async_trait]
impl AgentAdapter for CursorCloudAgentAdapter {
    fn id(&self) -> &str {
        ADAPTER_CURSOR_CLOUD_AGENT
    }

    async fn create_session(
        &self,
        input: CreateSessionInput,
        ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        let remote_url = resolve_remote_url(&input.cwd, &ctx.db, &input.project_id)?;
        let branch = crate::discovery::git_info::git_branch_with_timeout(
            std::path::Path::new(&input.cwd),
            std::time::Duration::from_secs(2),
        );

        let client = CursorCloudClient::new()?;

        let mut req = CreateAgentRequest {
            prompt: PromptBody {
                text: input.prompt.clone(),
            },
            model: ctx
                .cloud_settings
                .model
                .as_ref()
                .map(|id| client::ModelSelection { id: id.clone() }),
            repos: Some(vec![RepoConfig {
                url: remote_url.clone(),
                starting_ref: branch.clone(),
            }]),
            auto_create_pr: Some(ctx.cloud_settings.auto_create_pr),
        };

        (ctx.event_handler)(AgentEvent::AgentStatus {
            session_id: input.session_id.clone(),
            status: FridaySessionStatus::Starting,
            message: Some("Creating cloud agent".into()),
            timestamp: now_iso(),
        });

        (ctx.event_handler)(AgentEvent::AgentMessage {
            session_id: input.session_id.clone(),
            role: MessageRole::User,
            content: input.prompt.clone(),
            timestamp: now_iso(),
        });

        let resp = client.create_agent(req).await?;
        let agent_id = resp.agent.id.clone();
        let run_id = resp.run.id.clone();

        CloudAgentsRepo::new(&ctx.db).upsert(
            &input.session_id,
            &agent_id,
            Some(&run_id),
            None,
            Some(&resp.run.status),
        )?;

        let title = resp
            .agent
            .name
            .unwrap_or_else(|| format!("Cloud · {}", repo_display_name(&remote_url)));

        spawn_run_stream(
            input.session_id.clone(),
            agent_id.clone(),
            run_id.clone(),
            ctx.event_handler.clone(),
            ctx.db.clone(),
        );

        let project_id = input.project_id.clone();
        Ok(FridaySession {
            id: input.session_id,
            title,
            session_type: AgentSessionType::CursorCloud,
            ownership: SessionOwnership::Friday,
            adapter_id: ADAPTER_CURSOR_CLOUD_AGENT.into(),
            status: map_run_status(&resp.run.status),
            control_level: ControlLevel::Full,
            project_id: Some(project_id.clone()),
            prompt: Some(input.prompt),
            summary: None,
            repo: Some(crate::core::event::SessionRepo {
                id: project_id,
                name: repo_display_name(&remote_url),
                local_path: Some(input.cwd),
                remote_url: Some(remote_url),
                branch,
            }),
            process: None,
            cloud: Some(SessionCloud {
                agent_id: Some(agent_id),
                run_id: Some(run_id),
                pr_url: None,
                artifact_ids: None,
            }),
            created_at: now_iso(),
            started_at: Some(now_iso()),
            updated_at: now_iso(),
            completed_at: None,
        })
    }

    async fn attach_session(
        &self,
        _input: AttachSessionInput,
        _ctx: &AdapterContext,
    ) -> AppResult<FridaySession> {
        Err(AppError::Other(
            "cursor-cloud-agent does not support attach".into(),
        ))
    }

    async fn stop_session(&self, session_id: &str, ctx: &AdapterContext) -> AppResult<()> {
        let record = CloudAgentsRepo::new(&ctx.db).get_by_session(session_id)?;
        let client = CursorCloudClient::new()?;

        if let Some(run_id) = &record.run_id {
            client.cancel_run(&record.agent_id, run_id).await?;
        }

        CloudAgentsRepo::new(&ctx.db).update_run(session_id, record.run_id.as_deref().unwrap_or(""), Some("CANCELLED"))?;

        (ctx.event_handler)(AgentEvent::AgentStatus {
            session_id: session_id.to_string(),
            status: FridaySessionStatus::Stopped,
            message: Some("Cloud run cancelled".into()),
            timestamp: now_iso(),
        });

        Ok(())
    }

    async fn send_message(
        &self,
        session_id: &str,
        message: &str,
        ctx: &AdapterContext,
    ) -> AppResult<()> {
        let record = CloudAgentsRepo::new(&ctx.db).get_by_session(session_id)?;
        let client = CursorCloudClient::new()?;

        (ctx.event_handler)(AgentEvent::AgentMessage {
            session_id: session_id.to_string(),
            role: MessageRole::User,
            content: message.to_string(),
            timestamp: now_iso(),
        });

        let resp = client.create_run(&record.agent_id, message).await?;
        let run_id = resp.run.id.clone();

        CloudAgentsRepo::new(&ctx.db).update_run(session_id, &run_id, Some(&resp.run.status))?;

        spawn_run_stream(
            session_id.to_string(),
            record.agent_id.clone(),
            run_id,
            ctx.event_handler.clone(),
            ctx.db.clone(),
        );

        Ok(())
    }
}

fn resolve_remote_url(
    cwd: &str,
    db: &Arc<crate::storage::Database>,
    project_id: &str,
) -> AppResult<String> {
    use crate::storage::ProjectsRepo;

    if let Ok(project) = ProjectsRepo::new(db).get(project_id) {
        if let Some(url) = project.remote_url {
            if !url.is_empty() {
                return Ok(normalize_github_url(&url));
            }
        }
    }

    git_remote_origin_url(std::path::Path::new(cwd))
        .map(|u| normalize_github_url(&u))
        .ok_or_else(|| {
            AppError::Other(
                "No GitHub remote URL found. Add a project with git remote origin or set remote_url.".into(),
            )
        })
}

fn normalize_github_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("https://") || trimmed.starts_with("http://") {
        trimmed.trim_end_matches(".git").to_string()
    } else if trimmed.starts_with("git@github.com:") {
        let path = trimmed.trim_start_matches("git@github.com:");
        format!("https://github.com/{}", path.trim_end_matches(".git"))
    } else {
        trimmed.to_string()
    }
}

fn repo_display_name(remote_url: &str) -> String {
    remote_url
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .unwrap_or("cloud")
        .to_string()
}

pub fn resume_cloud_session(
    session: &FridaySession,
    handler: EventHandler,
    db: Arc<crate::storage::Database>,
) -> AppResult<()> {
    let cloud = session
        .cloud
        .as_ref()
        .ok_or_else(|| AppError::Other("missing cloud metadata".into()))?;
    let agent_id = cloud
        .agent_id
        .as_ref()
        .ok_or_else(|| AppError::Other("missing agent_id".into()))?;
    let run_id = cloud
        .run_id
        .as_ref()
        .ok_or_else(|| AppError::Other("missing run_id".into()))?;

    spawn_run_stream(
        session.id.clone(),
        agent_id.clone(),
        run_id.clone(),
        handler,
        db,
    );
    Ok(())
}
