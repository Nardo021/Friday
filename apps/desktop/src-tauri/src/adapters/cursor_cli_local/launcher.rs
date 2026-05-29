use std::path::{Path, PathBuf};

use crate::errors::{AppError, AppResult};
use crate::storage::settings_repo::CursorSettings;

#[derive(Debug, Clone)]
pub struct BuiltCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
}

pub fn resolve_executable(settings: &CursorSettings) -> String {
    settings
        .executable_path
        .clone()
        .unwrap_or_else(default_cursor_executable)
}

pub fn validate_executable(path: &str) -> AppResult<()> {
    let p = Path::new(path);
    if p.exists() {
        return Ok(());
    }
    if which_in_path(path) {
        return Ok(());
    }
    Err(AppError::Process(format!(
        "cursor executable not found: {path}"
    )))
}

pub fn build_command(
    prompt: &str,
    cwd: &str,
    settings: &CursorSettings,
) -> AppResult<BuiltCommand> {
    let executable = resolve_executable(settings);
    validate_executable(&executable)?;

    let mut args = Vec::new();
    for template in &settings.arg_templates.headless_stream {
        let rendered = template
            .replace("{prompt}", prompt)
            .replace("{cwd}", cwd)
            .replace(
                "{outputFormat}",
                &settings.default_output_format,
            );
        if !rendered.is_empty() {
            args.push(rendered);
        }
    }

    if args.is_empty() {
        args.push("-p".to_string());
        args.push("--output-format".to_string());
        args.push(settings.default_output_format.clone());
        if settings.default_output_format == "stream-json" {
            args.push("--stream-partial-output".to_string());
        }
        args.push(prompt.to_string());
    }

    Ok(BuiltCommand {
        executable,
        args,
        cwd: PathBuf::from(cwd),
        env: vec![],
    })
}

fn default_cursor_executable() -> String {
    if cfg!(target_os = "windows") {
        "cursor-agent.cmd".to_string()
    } else {
        "cursor-agent".to_string()
    }
}

fn which_in_path(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| dir.join(name).exists())
}
