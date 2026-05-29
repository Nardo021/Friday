use rusqlite::params;
use uuid::Uuid;

use crate::core::event::{Project, now_iso};
use crate::discovery::git_info::git_remote_origin_url;
use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

pub struct ProjectsRepo<'a> {
    db: &'a Database,
}

impl<'a> ProjectsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn add(&self, name: &str, path: &str, trusted: bool) -> AppResult<Project> {
        let remote_url =
            git_remote_origin_url(std::path::Path::new(path));
        let project = Project {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            path: path.to_string(),
            project_type: None,
            remote_url,
            trusted,
            default_adapter_id: "cursor-cli".into(),
            created_at: now_iso(),
            last_used_at: now_iso(),
        };

        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO projects (id, name, path, project_type, remote_url, trusted, default_adapter_id, created_at, last_used_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    project.id,
                    project.name,
                    project.path,
                    project.project_type,
                    project.remote_url,
                    project.trusted as i32,
                    project.default_adapter_id,
                    project.created_at,
                    project.last_used_at,
                ],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(project)
        })
    }

    pub fn list(&self) -> AppResult<Vec<Project>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, name, path, project_type, remote_url, trusted, default_adapter_id, created_at, last_used_at
                     FROM projects ORDER BY last_used_at DESC",
                )
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        project_type: row.get(3)?,
                        remote_url: row.get(4)?,
                        trusted: row.get::<_, i32>(5)? != 0,
                        default_adapter_id: row.get(6)?,
                        created_at: row.get(7)?,
                        last_used_at: row.get(8)?,
                    })
                })
                .map_err(|e| AppError::Storage(e.to_string()))?;

            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|e| AppError::Storage(e.to_string()))
        })
    }

    pub fn get(&self, id: &str) -> AppResult<Project> {
        self.db.with_conn(|conn| {
            conn.query_row(
                "SELECT id, name, path, project_type, remote_url, trusted, default_adapter_id, created_at, last_used_at
                 FROM projects WHERE id = ?1",
                params![id],
                |row| {
                    Ok(Project {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        path: row.get(2)?,
                        project_type: row.get(3)?,
                        remote_url: row.get(4)?,
                        trusted: row.get::<_, i32>(5)? != 0,
                        default_adapter_id: row.get(6)?,
                        created_at: row.get(7)?,
                        last_used_at: row.get(8)?,
                    })
                },
            )
            .map_err(|_| AppError::ProjectNotFound(id.to_string()))
        })
    }

    pub fn touch(&self, id: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE projects SET last_used_at = ?1 WHERE id = ?2",
                params![now_iso(), id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn update_remote_url(&self, id: &str, remote_url: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute(
                "UPDATE projects SET remote_url = ?1 WHERE id = ?2",
                params![remote_url, id],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }

    pub fn refresh_remote_url_from_git(&self, id: &str) -> AppResult<Option<String>> {
        let project = self.get(id)?;
        if let Some(url) = git_remote_origin_url(std::path::Path::new(&project.path)) {
            self.update_remote_url(id, &url)?;
            Ok(Some(url))
        } else {
            Ok(project.remote_url)
        }
    }

    /// Default workspace when the user has not picked a repo (chat-first).
    pub fn get_or_create_general_workspace(&self) -> AppResult<Project> {
        let path = dirs::home_dir()
            .ok_or_else(|| AppError::Other("cannot resolve home directory".into()))?;
        let path_str = path.to_string_lossy().to_string();
        for project in self.list()? {
            if project.path == path_str {
                return Ok(project);
            }
        }
        self.add("General", &path_str, true)
    }
}
