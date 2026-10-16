use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Config {
    /// Default dongle ID used when a command does not specify one.
    pub default_dongle_id: Option<String>,

    /// Comma developer JWT token.
    /// Prefer the `COMMA_TOKEN` environment variable; this field is a fallback
    /// and should be used with care (config files may be world-readable).
    pub token: Option<String>,
}

impl Config {
    pub const APP_NAME: &'static str = "comma-cli";

    pub fn load() -> anyhow::Result<Self> {
        Ok(confy::load(Self::APP_NAME, None)?)
    }

    pub fn store(&self) -> anyhow::Result<()> {
        Ok(confy::store(Self::APP_NAME, None, self)?)
    }

    pub fn path() -> anyhow::Result<std::path::PathBuf> {
        Ok(confy::get_configuration_file_path(Self::APP_NAME, None)?)
    }

    /// Resolve the token: environment variable takes precedence over config.
    pub fn resolve_token(&self) -> anyhow::Result<String> {
        if let Ok(token) = std::env::var("COMMA_TOKEN")
            && !token.is_empty()
        {
            return Ok(token);
        }
        self.token
            .clone()
            .filter(|t| !t.is_empty())
            .ok_or_else(|| anyhow::anyhow!(
                "No token found. Set the COMMA_TOKEN environment variable or store one in the config."
            ))
    }

    /// Resolve the default dongle ID: `COMMA_DONGLE` env var takes precedence over config.
    pub fn resolve_dongle(&self) -> Option<String> {
        if let Ok(id) = std::env::var("COMMA_DONGLE")
            && !id.is_empty()
        {
            return Some(id);
        }
        self.default_dongle_id.clone().filter(|s| !s.is_empty())
    }
}
