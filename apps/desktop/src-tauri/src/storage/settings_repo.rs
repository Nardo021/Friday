use rusqlite::params;
use serde::{Deserialize, Serialize};

use crate::errors::{AppError, AppResult};
use crate::security::SecretStore;
use crate::storage::sqlite::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSettings {
    #[serde(default = "default_auto_create_pr")]
    pub auto_create_pr: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

fn default_auto_create_pr() -> bool {
    true
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            auto_create_pr: true,
            model: None,
        }
    }
}

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
    #[serde(default, skip_deserializing)]
    pub api_key_configured: bool,
    pub default_mode: String,
    pub default_output_format: String,
    #[serde(default = "default_use_pty")]
    pub use_pty: bool,
    #[serde(default)]
    pub arg_templates: Vec<String>,
    #[serde(default = "default_terminal_cols")]
    pub terminal_cols: u16,
    #[serde(default = "default_terminal_rows")]
    pub terminal_rows: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnboardingSettings {
    #[serde(default)]
    pub completed: bool,
}

impl Default for OnboardingSettings {
    fn default() -> Self {
        Self { completed: false }
    }
}

fn default_terminal_cols() -> u16 {
    120
}

fn default_terminal_rows() -> u16 {
    30
}

fn default_use_pty() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_x: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_y: Option<f64>,
    #[serde(default = "default_patrol_enabled")]
    pub patrol_enabled: bool,
}

fn default_patrol_enabled() -> bool {
    true
}

impl Default for PetSettings {
    fn default() -> Self {
        Self {
            last_x: None,
            last_y: None,
            patrol_enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSettings {
    pub push_to_talk: bool,
    pub confirm_before_send: bool,
    pub auto_send_after_transcription: bool,
    #[serde(default = "default_transcription_language")]
    pub transcription_language: String,
    #[serde(default, skip_deserializing)]
    pub stt_api_key_configured: bool,
}

fn default_transcription_language() -> String {
    "en".into()
}

impl Default for VoiceSettings {
    fn default() -> Self {
        Self {
            push_to_talk: false,
            confirm_before_send: true,
            auto_send_after_transcription: false,
            transcription_language: default_transcription_language(),
            stt_api_key_configured: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MobileBridgeSettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_bridge_port")]
    pub port: u16,
    #[serde(default)]
    pub auth_token: String,
}

fn default_bridge_port() -> u16 {
    8787
}

impl Default for MobileBridgeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_bridge_port(),
            auth_token: String::new(),
        }
    }
}

pub fn ensure_mobile_bridge_token(settings: &mut FridaySettings) {
    if settings.mobile_bridge.auth_token.is_empty() {
        settings.mobile_bridge.auth_token = uuid::Uuid::new_v4().to_string();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutSettings {
    pub quick_bubble: String,
    pub open_panel: String,
    pub voice_input: String,
    pub stop_session: String,
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            quick_bubble: "CommandOrControl+Space".into(),
            open_panel: "CommandOrControl+Shift+F".into(),
            voice_input: "CommandOrControl+Shift+V".into(),
            stop_session: "CommandOrControl+Period".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FridaySettings {
    pub appearance: AppearanceSettings,
    pub behavior: BehaviorSettings,
    pub security: SecuritySettings,
    pub cursor: CursorSettings,
    #[serde(default)]
    pub onboarding: OnboardingSettings,
    #[serde(default)]
    pub pet: PetSettings,
    #[serde(default)]
    pub voice: VoiceSettings,
    #[serde(default)]
    pub shortcuts: ShortcutSettings,
    #[serde(default)]
    pub cloud: CloudSettings,
    #[serde(default)]
    pub mobile_bridge: MobileBridgeSettings,
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
                api_key_configured: false,
                default_mode: "headless".into(),
                default_output_format: "stream-json".into(),
                use_pty: true,
                arg_templates: vec![],
                terminal_cols: default_terminal_cols(),
                terminal_rows: default_terminal_rows(),
            },
            onboarding: OnboardingSettings::default(),
            pet: PetSettings::default(),
            voice: VoiceSettings::default(),
            shortcuts: ShortcutSettings::default(),
            cloud: CloudSettings::default(),
            mobile_bridge: MobileBridgeSettings::default(),
        }
    }
}

const CURSOR_API_KEY_SETTING: &str = "cursor_api_key";

pub struct SettingsRepo<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn migrate_secrets(&self) -> AppResult<()> {
        if let Some(legacy) = self.read_sqlite_secret(CURSOR_API_KEY_SETTING)? {
            if !SecretStore::has_cursor_api_key()? {
                SecretStore::save_cursor_api_key(&legacy)?;
            }
            self.delete_sqlite_secret(CURSOR_API_KEY_SETTING)?;
        }
        Ok(())
    }

    pub fn get(&self) -> AppResult<FridaySettings> {
        let mut settings = self.db.with_conn(|conn| {
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
        })?;
        settings.cursor.api_key_configured = SecretStore::has_cursor_api_key()?;
        settings.voice.stt_api_key_configured = SecretStore::has_stt_api_key()?;
        ensure_mobile_bridge_token(&mut settings);
        Ok(settings)
    }

    pub fn save(&self, settings: &FridaySettings) -> AppResult<()> {
        let mut to_save = settings.clone();
        to_save.cursor.api_key_configured = SecretStore::has_cursor_api_key()?;
        to_save.voice.stt_api_key_configured = SecretStore::has_stt_api_key()?;
        let json = serde_json::to_string(&to_save)?;
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

    pub fn save_cursor_api_key(&self, key: &str) -> AppResult<()> {
        SecretStore::save_cursor_api_key(key)?;
        self.delete_sqlite_secret(CURSOR_API_KEY_SETTING)?;
        Ok(())
    }

    pub fn clear_cursor_api_key(&self) -> AppResult<()> {
        SecretStore::clear_cursor_api_key()?;
        self.delete_sqlite_secret(CURSOR_API_KEY_SETTING)?;
        Ok(())
    }

    pub fn has_cursor_api_key(&self) -> AppResult<bool> {
        if SecretStore::has_cursor_api_key()? {
            return Ok(true);
        }
        if self.read_sqlite_secret(CURSOR_API_KEY_SETTING)?.is_some() {
            self.migrate_secrets()?;
            return Ok(true);
        }
        Ok(false)
    }

    fn read_sqlite_secret(&self, key: &str) -> AppResult<Option<String>> {
        self.db.with_conn(|conn| {
            let mut stmt = conn
                .prepare("SELECT value_json FROM settings WHERE key = ?1")
                .map_err(|e| AppError::Storage(e.to_string()))?;
            let mut rows = stmt
                .query(params![key])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            if let Some(row) = rows.next().map_err(|e| AppError::Storage(e.to_string()))? {
                let value: String = row.get(0).map_err(|e| AppError::Storage(e.to_string()))?;
                Ok(Some(value))
            } else {
                Ok(None)
            }
        })
    }

    fn delete_sqlite_secret(&self, key: &str) -> AppResult<()> {
        self.db.with_conn(|conn| {
            conn.execute("DELETE FROM settings WHERE key = ?1", params![key])
                .map_err(|e| AppError::Storage(e.to_string()))?;
            Ok(())
        })
    }
}
