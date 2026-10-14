//! Shared world and helpers for comma-rs cucumber tests.

use comma_connect_api::{Client, RouteFiles};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// World shared across steps.
#[derive(Debug, Default, cucumber::World)]
#[world(init = Self::default)]
pub struct World {
    pub client: Option<Client>,
    pub last_json: Option<serde_json::Value>,
    pub last_files: Option<RouteFiles>,
    pub last_error: Option<String>,
}

/// Load token and dongle from the environment (or a `.env` file if present).
pub fn load_test_env() {
    let candidates = [
        std::path::PathBuf::from(".env"),
        std::path::PathBuf::from("../.env"),
        std::path::PathBuf::from("../../.env"),
        std::path::PathBuf::from("../../../.env"),
    ];
    for path in candidates {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((k, v)) = line.split_once('=') {
                    let k = k.trim();
                    let v = v.trim().trim_matches('"').trim_matches('\'');
                    if std::env::var(k).is_err() {
                        // SAFETY: tests are single-threaded with respect to env mutation.
                        unsafe { std::env::set_var(k, v); }
                    }
                }
            }
            return;
        }
    }
}

pub fn require_token() -> Result<String> {
    std::env::var("COMMA_TOKEN")
        .map_err(|_| "COMMA_TOKEN not set (export it or put it in .env)".into())
}

pub fn require_dongle() -> Result<String> {
    std::env::var("COMMA_DONGLE")
        .map_err(|_| "COMMA_DONGLE not set (export it or put it in .env)".into())
}
