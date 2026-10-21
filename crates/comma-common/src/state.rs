//! Application state directory and last-used device persistence.
//!
//! State lives under `$XDG_STATE_HOME/comma` or `~/.local/state/comma`.
//! Last-used device ID for the TUI is stored in a plain file
//! `tui-last-used-device-id`.

use std::fs;
use std::path::PathBuf;

/// Directory for logs and other persistent state.
pub fn state_dir() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_STATE_HOME")
        && !xdg.is_empty()
    {
        return PathBuf::from(xdg).join("comma");
    }
    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home).join(".local/state/comma");
    }
    PathBuf::from(".local/state/comma")
}

const LAST_DEVICE_FILE: &str = "tui-last-used-device-id";

/// Read the last-used dongle ID stored by the TUI, if any.
pub fn load_last_dongle() -> Option<String> {
    let path = state_dir().join(LAST_DEVICE_FILE);
    fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Persist the last-used dongle ID for the TUI.
pub fn store_last_dongle(dongle_id: &str) -> anyhow::Result<()> {
    let dir = state_dir();
    fs::create_dir_all(&dir)?;
    let path = dir.join(LAST_DEVICE_FILE);
    fs::write(path, dongle_id)?;
    Ok(())
}

/// Resolve the active dongle for the TUI according to the agreed order:
/// CLI flag > COMMA_DONGLE env > last-used state file > config default.
pub fn resolve_tui_dongle(
    cli_dongle: Option<&str>,
    config_default: Option<&str>,
) -> Option<String> {
    if let Some(d) = cli_dongle.filter(|s| !s.is_empty()) {
        return Some(d.to_string());
    }
    if let Ok(id) = std::env::var("COMMA_DONGLE")
        && !id.is_empty()
    {
        return Some(id);
    }
    if let Some(last) = load_last_dongle() {
        return Some(last);
    }
    config_default
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_dir_contains_comma() {
        let dir = state_dir();
        assert!(dir.ends_with("comma"));
    }

    #[test]
    fn resolve_prefers_cli() {
        let r = resolve_tui_dongle(Some("cli"), Some("cfg"));
        assert_eq!(r.as_deref(), Some("cli"));
    }
}
