use tauri::State;

use crate::core::event::Project;
use crate::core::AgentCore;
use crate::storage::ProjectsRepo;

#[tauri::command]
pub fn add_project(
    core: State<'_, AgentCore>,
    name: String,
    path: String,
    trusted: bool,
) -> Result<Project, crate::errors::AppError> {
    ProjectsRepo::new(&core.db).add(&name, &path, trusted)
}

#[tauri::command]
pub fn list_projects(core: State<'_, AgentCore>) -> Result<Vec<Project>, crate::errors::AppError> {
    ProjectsRepo::new(&core.db).list()
}

#[tauri::command]
pub fn get_project(
    core: State<'_, AgentCore>,
    project_id: String,
) -> Result<Project, crate::errors::AppError> {
    ProjectsRepo::new(&core.db).get(&project_id)
}
