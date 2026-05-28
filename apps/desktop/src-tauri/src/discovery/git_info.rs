use std::path::Path;
use std::process::Command;
use std::time::Duration;

use crate::errors::{AppError, AppResult};

pub fn git_remote_origin_url(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        None
    } else {
        Some(url)
    }
}

pub fn git_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if branch.is_empty() || branch == "HEAD" {
        None
    } else {
        Some(branch)
    }
}

pub fn git_branch_with_timeout(cwd: &Path, timeout: Duration) -> Option<String> {
    let cwd = cwd.to_path_buf();
    std::thread::scope(|s| {
        let handle = s.spawn(|| git_branch(&cwd));
        let start = std::time::Instant::now();
        while !handle.is_finished() {
            if start.elapsed() > timeout {
                return None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        handle.join().ok().flatten()
    })
}

pub fn repo_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}

pub fn match_project_id(
    cwd: &Path,
    projects: &[(String, String)],
) -> Option<(String, String)> {
    let cwd = std::fs::canonicalize(cwd).ok()?;
    let mut best: Option<(String, String, usize)> = None;

    for (id, project_path) in projects {
        if let Ok(canonical) = std::fs::canonicalize(project_path) {
            if cwd.starts_with(&canonical) {
                let len = canonical.as_os_str().len();
                if best.as_ref().map(|(_, _, l)| len > *l).unwrap_or(true) {
                    best = Some((id.clone(), repo_name_from_path(&canonical), len));
                }
            }
        }
    }

    best.map(|(id, name, _)| (id, name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_name_from_path_works() {
        assert_eq!(
            repo_name_from_path(Path::new("/tmp/my-repo")),
            "my-repo"
        );
    }
}
