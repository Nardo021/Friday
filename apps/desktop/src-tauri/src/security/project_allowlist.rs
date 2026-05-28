use std::path::Path;

use crate::core::event::Project;
use crate::errors::{AppError, AppResult};
use crate::storage::projects_repo::ProjectsRepo;

pub struct ProjectAllowlist;

impl ProjectAllowlist {
    pub fn validate_path(projects_repo: &ProjectsRepo, cwd: &str) -> AppResult<Project> {
        let normalized = normalize_path(cwd);
        let projects = projects_repo.list()?;

        projects
            .into_iter()
            .find(|p| normalize_path(&p.path) == normalized)
            .ok_or_else(|| AppError::ProjectNotAllowed(normalized))
    }

    pub fn is_trusted(project: &Project) -> bool {
        project.trusted
    }
}

fn normalize_path(path: &str) -> String {
    Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.replace('\\', "/").trim_end_matches('/').to_string())
}
