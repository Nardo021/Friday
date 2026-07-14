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

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorCliProbe {
    pub found: bool,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn resolve_executable(settings: &CursorSettings) -> String {
    settings
        .executable_path
        .clone()
        .unwrap_or_else(default_cursor_executable)
}

pub fn validate_executable(path: &str) -> AppResult<()> {
    let p = Path::new(path);
    if p.is_file() {
        return Ok(());
    }
    if which_in_path(path).is_some() {
        return Ok(());
    }
    Err(AppError::Process(format!(
        "cursor-agent not found: {path}. Install Cursor CLI and ensure it is on PATH, or set the executable path in Settings."
    )))
}

pub fn probe_executable(settings: &CursorSettings) -> CursorCliProbe {
    let path = resolve_executable(settings);
    match validate_executable(&path) {
        Ok(()) => {
            let resolved = which_in_path(&path)
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| path.clone());
            CursorCliProbe {
                found: true,
                path: resolved,
                error: None,
            }
        }
        Err(e) => CursorCliProbe {
            found: false,
            path,
            error: Some(e.to_string()),
        },
    }
}

pub fn build_command(
    prompt: &str,
    cwd: &str,
    settings: &CursorSettings,
) -> AppResult<BuiltCommand> {
    let executable = resolve_executable(settings);
    validate_executable(&executable)?;

    let templates = &settings.arg_templates.headless_stream;
    let had_prompt_placeholder = templates.iter().any(|t| t.contains("{prompt}"));

    let mut args = Vec::new();
    for template in templates {
        let rendered = template
            .replace("{prompt}", prompt)
            .replace("{cwd}", cwd)
            .replace("{outputFormat}", &settings.default_output_format);
        if !rendered.is_empty() {
            args.push(rendered);
        }
    }

    if args.is_empty() {
        args.push("--print".to_string());
        args.push("--output-format".to_string());
        args.push(settings.default_output_format.clone());
        if settings.default_output_format == "stream-json" {
            args.push("--stream-partial-output".to_string());
        }
        args.push(prompt.to_string());
    } else if !had_prompt_placeholder && !args.iter().any(|a| a == prompt) {
        // Templates without `{prompt}` previously dropped the task entirely.
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

fn which_in_path(name: &str) -> Option<PathBuf> {
    let path = Path::new(name);
    if path.is_file() {
        return Some(path.to_path_buf());
    }
    let path_var = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path_var) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
        #[cfg(target_os = "windows")]
        {
            let with_cmd = dir.join(format!("{name}.cmd"));
            if with_cmd.is_file() {
                return Some(with_cmd);
            }
            let with_exe = dir.join(format!("{name}.exe"));
            if with_exe.is_file() {
                return Some(with_exe);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::settings_repo::{CursorArgTemplates, CursorSettings};

    fn settings_with_templates(templates: Vec<&str>) -> CursorSettings {
        CursorSettings {
            executable_path: Some("/bin/true".into()),
            api_key_configured: false,
            default_mode: "headless".into(),
            default_output_format: "stream-json".into(),
            use_pty: true,
            arg_templates: CursorArgTemplates {
                headless_stream: templates.into_iter().map(String::from).collect(),
            },
            terminal_cols: 120,
            terminal_rows: 30,
        }
    }

    #[test]
    fn appends_prompt_when_templates_omit_placeholder() {
        let settings = settings_with_templates(vec![
            "--print",
            "--output-format",
            "stream-json",
        ]);
        let built = build_command("fix the bug", "/tmp", &settings).unwrap();
        assert_eq!(built.args.last().map(String::as_str), Some("fix the bug"));
    }

    #[test]
    fn substitutes_prompt_placeholder() {
        let settings = settings_with_templates(vec![
            "--print",
            "--output-format",
            "{outputFormat}",
            "--stream-partial-output",
            "{prompt}",
        ]);
        let built = build_command("hello", "/repo", &settings).unwrap();
        assert!(built.args.contains(&"stream-json".to_string()));
        assert!(built.args.contains(&"hello".to_string()));
        assert!(!built.args.iter().any(|a| a.contains("{prompt}")));
    }
}
