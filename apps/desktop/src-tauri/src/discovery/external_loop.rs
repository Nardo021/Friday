use std::collections::HashSet;
use std::sync::Arc;

use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use uuid::Uuid;

use crate::adapters::registry::ADAPTER_EXTERNAL_CURSOR_OBSERVER;
use crate::adapters::AttachSessionInput;
use crate::core::event::{AgentEvent, FridaySessionStatus, SessionOwnership, now_iso};
use crate::core::AgentCore;
use crate::discovery::process_util::process_alive;
use crate::discovery::scan_cursor_agent_processes_with;
use crate::storage::SessionsRepo;

const DISCOVERY_INTERVAL_SECS: u64 = 8;

pub fn start_discovery_loop(core: Arc<AgentCore>) {
    tauri::async_runtime::spawn(async move {
        discovery_loop(&core).await;
    });
}

async fn discovery_loop(core: &AgentCore) {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(DISCOVERY_INTERVAL_SECS)).await;

        let app = match core.app_handle.lock().await.clone() {
            Some(a) => a,
            None => continue,
        };

        let (mut external_pids, has_external) = {
            let mgr = core.session_manager.lock().await;
            let pids: HashSet<u32> = mgr
                .list()
                .iter()
                .filter_map(|s| s.process.as_ref().and_then(|p| p.pid))
                .collect();
            let has_external = mgr.list().iter().any(|s| s.ownership == SessionOwnership::External);
            (pids, has_external)
        };

        if has_external {
            cleanup_dead_external_sessions(core, &mut system).await;
        }

        let discovered = scan_cursor_agent_processes_with(&mut system);
        if discovered.is_empty() {
            continue;
        }

        let settings = core.settings_snapshot();
        let ctx = core.build_context(settings.cursor.clone(), settings.cloud.clone());

        let adapter = match core
            .adapter_registry
            .get_adapter(ADAPTER_EXTERNAL_CURSOR_OBSERVER)
        {
            Ok(a) => a,
            Err(_) => continue,
        };

        for proc in discovered {
            if external_pids.contains(&proc.pid) {
                continue;
            }

            let session_id = Uuid::new_v4().to_string();

            let mut session = match adapter
                .attach_session(
                    AttachSessionInput {
                        session_id: session_id.clone(),
                        pid: proc.pid,
                        cwd: proc.cwd.clone(),
                        exe_name: Some(proc.exe_name),
                    },
                    &ctx,
                )
                .await
            {
                Ok(s) => s,
                Err(_) => continue,
            };

            if let Some(cwd) = proc.cwd.as_deref() {
                if let Ok(Some((project_id, repo))) = core.match_external_project(cwd) {
                    if !project_id.is_empty() {
                        session.project_id = Some(project_id);
                    }
                    session.repo = Some(repo);
                    session.title = format!(
                        "External CLI · {}",
                        session
                            .repo
                            .as_ref()
                            .map(|r| r.name.as_str())
                            .unwrap_or("unknown")
                    );
                }
            }

            core.session_manager.lock().await.create(session.clone());
            if SessionsRepo::new(&core.db).upsert(&session).is_err() {
                continue;
            }
            external_pids.insert(proc.pid);

            let event = AgentEvent::SessionDiscovered {
                session_id: session_id.clone(),
                source: crate::core::event::DiscoverySource::ProcessScan,
                timestamp: now_iso(),
            };
            let _ = core.dispatch_event(app.clone(), event).await;
        }
    }
}

async fn cleanup_dead_external_sessions(core: &AgentCore, system: &mut System) {
    let to_stop: Vec<String> = {
        let sessions = core.session_manager.lock().await.list();
        sessions
            .into_iter()
            .filter(|s| s.ownership == SessionOwnership::External)
            .filter_map(|s| {
                let pid = s.process.as_ref().and_then(|p| p.pid)?;
                if process_alive(system, pid) {
                    None
                } else {
                    Some(s.id)
                }
            })
            .collect()
    };

    if to_stop.is_empty() {
        return;
    }

    let mut mgr = core.session_manager.lock().await;
    for sid in to_stop {
        let _ = mgr.update_status(&sid, FridaySessionStatus::Stopped);
        let _ = SessionsRepo::new(&core.db).update_status(&sid, FridaySessionStatus::Stopped, None);
    }
}
