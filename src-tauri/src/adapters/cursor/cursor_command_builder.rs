use std::path::PathBuf;

use crate::errors::AppResult;
use crate::storage::settings_repo::CursorSettings;

#[derive(Debug, Clone)]
pub struct BuiltCommand {
    pub executable: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
}

pub struct CursorCommandBuilder;

impl CursorCommandBuilder {
    pub fn build(prompt: &str, cwd: &str, settings: &CursorSettings) -> AppResult<BuiltCommand> {
        let executable = settings
            .executable_path
            .clone()
            .unwrap_or_else(default_cursor_executable);

        let mut args = vec![
            "-p".to_string(),
            "--output-format".to_string(),
            settings.default_output_format.clone(),
        ];

        if settings.default_output_format == "stream-json" {
            args.push("--stream-partial-output".to_string());
        }

        args.push(prompt.to_string());

        Ok(BuiltCommand {
            executable,
            args,
            cwd: PathBuf::from(cwd),
            env: vec![],
        })
    }
}

fn default_cursor_executable() -> String {
    if cfg!(target_os = "windows") {
        "cursor-agent.cmd".to_string()
    } else {
        "cursor-agent".to_string()
    }
}
