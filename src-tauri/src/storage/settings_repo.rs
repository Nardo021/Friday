use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::storage::sqlite::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppearanceSettings {
    pub theme: String,
    pub accent_color: String,
    pub pet_scale: f64,
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BehaviorSettings {
    pub launch_at_startup: bool,
    pub always_on_top: bool,
    pub show_bubble_on_status_change: bool,
    pub auto_collapse_bubble: bool,
    pub sound_effects: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecuritySettings {
    pub require_approval_for_high_risk_commands: bool,
    pub require_approval_for_medium_risk_commands: bool,
    pub redact_secrets: bool,
    pub allow_shell_commands: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CursorSettings {
    pub executable_path: Option<String>,
    pub default_mode: String,
    pub default_output_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FridaySettings {
    pub appearance: AppearanceSettings,
    pub behavior: BehaviorSettings,
    pub security: SecuritySettings,
    pub cursor: CursorSettings,
}

impl Default for FridaySettings {
    fn default() -> Self {
        Self {
            appearance: AppearanceSettings {
                theme: "system".into(),
                accent_color: "#6366f1".into(),
                pet_scale: 1.0,
                reduced_motion: false,
            },
            behavior: BehaviorSettings {
                launch_at_startup: false,
                always_on_top: false,
                show_bubble_on_status_change: true,
                auto_collapse_bubble: true,
                sound_effects: false,
            },
            security: SecuritySettings {
                require_approval_for_high_risk_commands: true,
                require_approval_for_medium_risk_commands: false,
                redact_secrets: true,
                allow_shell_commands: true,
            },
            cursor: CursorSettings {
                executable_path: None,
                default_mode: "headless".into(),
                default_output_format: "stream-json".into(),
            },
        }
    }
}

pub struct SettingsRepo<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn get(&self) -> AppResult<FridaySettings> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT value_json FROM settings WHERE key = 'friday'")
                .map_err(|e| AppError::Storage(e.to_string()))?;

            let mut rows = stmt
                .query([])
                .map_err(|e| AppError::Storage(e.to_string()))?;

            if let Some(row) = rows.next().map_err(|e| AppError::Storage(e.to_string()))? {
                let json: String = row.get(0).map_err(|e| AppError::Storage(e.to_string()))?;
                serde_json::from_str(&json).map_err(AppError::from)
            } else {
                Ok(FridaySettings::default())
            }
        })
    }

    pub fn save(&self, settings: &FridaySettings) -> AppResult<()> {
        let json = serde_json::to_string(settings)?;
        self.db.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value_json) VALUES ('friday', ?1)
                 ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json",
                params![json],
            )
            .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }
}
