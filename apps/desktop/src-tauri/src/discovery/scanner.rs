use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

#[derive(Debug, Clone)]
pub struct DiscoveredProcess {
    pub pid: u32,
    pub exe_name: String,
    pub cwd: Option<String>,
}

pub fn scan_cursor_agent_processes() -> Vec<DiscoveredProcess> {
    let mut system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );
    scan_cursor_agent_processes_with(&mut system)
}

pub fn scan_cursor_agent_processes_with(system: &mut System) -> Vec<DiscoveredProcess> {
    system.refresh_processes(ProcessesToUpdate::All, false);

    let mut found = Vec::new();
    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy();
        let name_lower = name.to_lowercase();
        let is_cursor_agent = name.eq_ignore_ascii_case("cursor-agent")
            || name.eq_ignore_ascii_case("cursor-agent.cmd")
            || name_lower.contains("cursor-agent");

        if !is_cursor_agent {
            continue;
        }

        let cwd = process.cwd().map(|p| p.to_string_lossy().to_string());

        found.push(DiscoveredProcess {
            pid: pid.as_u32(),
            exe_name: name.into_owned(),
            cwd,
        });
    }
    found
}
