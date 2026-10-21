//! Shared configuration for CLI and TUI.
//!
//! Config lives at the platform-standard location for app name "comma"
//! (typically `~/.config/comma/config.toml` on Linux).
//!
//! Structure:
//! ```toml
//! [common]
//! default_dongle_id = "..."
//! token = "..."
//!
//! [tui]
//! # TUI-specific settings (future)
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    #[serde(default)]
    pub common: CommonConfig,

    #[serde(default)]
    pub tui: TuiSection,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CommonConfig {
    /// Default dongle ID used when neither CLI flag, env, nor last-used state is set.
    pub default_dongle_id: Option<String>,

    /// Comma developer JWT token.
    /// Prefer the `COMMA_TOKEN` environment variable; this field is a fallback.
    pub token: Option<String>,
}

/// TUI-specific configuration section. Currently empty; reserved for future use.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct TuiSection {
    // Future TUI-only settings go here.
}

impl Config {
    pub const APP_NAME: &'static str = "comma";

    pub fn load() -> anyhow::Result<Self> {
        Ok(confy::load(Self::APP_NAME, Some("config"))?)
    }

    pub fn store(&self) -> anyhow::Result<()> {
        Ok(confy::store(Self::APP_NAME, Some("config"), self)?)
    }

    pub fn path() -> anyhow::Result<std::path::PathBuf> {
        Ok(confy::get_configuration_file_path(Self::APP_NAME, Some("config"))?)
    }

    /// Resolve token: CLI flag > env > config.
    pub fn resolve_token(&self, cli_token: Option<&str>) -> anyhow::Result<String> {
        if let Some(t) = cli_token.filter(|s| !s.is_empty()) {
            return Ok(t.to_string());
        }
        if let Ok(token) = std::env::var("COMMA_TOKEN")
            && !token.is_empty()
        {
            return Ok(token);
        }
        self.common
            .token
            .clone()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No token found. Pass --token, set the COMMA_TOKEN environment variable, \
                     or store one in the config file (common.token).\n\
                     Obtain a JWT at https://useradmin.comma.ai (click \"Get JWT\")."
                )
            })
    }

    /// Resolve default dongle from config only (env and CLI are handled by callers).
    pub fn default_dongle(&self) -> Option<String> {
        self.common
            .default_dongle_id
            .clone()
            .filter(|s| !s.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_prefers_cli() {
        let cfg = Config {
            common: CommonConfig {
                token: Some("cfg".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(cfg.resolve_token(Some("cli")).unwrap(), "cli");
    }

    #[test]
    fn missing_token_mentions_useradmin() {
        // Isolate from any ambient COMMA_TOKEN so the test is deterministic.
        let previous = std::env::var("COMMA_TOKEN").ok();
        // Safety: single-threaded test; we restore the previous value immediately after.
        unsafe { std::env::remove_var("COMMA_TOKEN"); }
        let cfg = Config::default();
        let result = cfg.resolve_token(None);
        // Restore before asserting so a panic doesn't leave the env dirty for other tests.
        if let Some(prev) = previous {
            unsafe { std::env::set_var("COMMA_TOKEN", prev); }
        }
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("token"));
        assert!(msg.contains("useradmin.comma.ai"));
    }
}
