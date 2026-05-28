use crate::core::event::RiskLevel;

pub fn classify_command_risk(command: &str) -> RiskLevel {
    let lower = command.to_lowercase();

    const HIGH_PATTERNS: &[&str] = &[
        "rm -rf",
        "rm -r",
        "del /s",
        "format ",
        "curl ",
        "| bash",
        "iex ",
        "git reset --hard",
        "git clean -fd",
        ".env",
        "drop table",
        "truncate ",
    ];

    const MEDIUM_PATTERNS: &[&str] = &[
        "pnpm install",
        "npm install",
        "yarn install",
        "git checkout",
        "git pull",
        "git push",
        "pip install",
        "cargo install",
    ];

    for pattern in HIGH_PATTERNS {
        if lower.contains(pattern) {
            return RiskLevel::High;
        }
    }

    for pattern in MEDIUM_PATTERNS {
        if lower.contains(pattern) {
            return RiskLevel::Medium;
        }
    }

    RiskLevel::Low
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_risk_rm_rf() {
        assert_eq!(classify_command_risk("rm -rf dist"), RiskLevel::High);
    }

    #[test]
    fn low_risk_test() {
        assert_eq!(classify_command_risk("pnpm test"), RiskLevel::Low);
    }

    #[test]
    fn medium_risk_install() {
        assert_eq!(classify_command_risk("pnpm install"), RiskLevel::Medium);
    }
}
