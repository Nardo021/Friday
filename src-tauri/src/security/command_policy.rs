use crate::core::event::RiskLevel;
use crate::storage::settings_repo::SecuritySettings;

pub struct CommandPolicy;

impl CommandPolicy {
    pub fn requires_approval(command: &str, settings: &SecuritySettings) -> bool {
        let risk = crate::security::risk_classifier::classify_command_risk(command);
        match risk {
            RiskLevel::High => settings.require_approval_for_high_risk_commands,
            RiskLevel::Medium => settings.require_approval_for_medium_risk_commands,
            RiskLevel::Low => false,
        }
    }
}
