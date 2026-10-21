//! TUI configuration and dongle resolution.
//!
//! Uses the shared config from `comma-common` (`~/.config/comma/config.toml`).
//! Last-used dongle is stored in the state directory, not in config.

pub use comma_common::{resolve_tui_dongle, store_last_dongle, Config};

/// Resolve the active dongle for this TUI session.
/// Order: CLI flag > COMMA_DONGLE > last-used state file > config default.
pub fn resolve_active_dongle(cfg: &Config, cli_dongle: Option<&str>) -> Option<String> {
    resolve_tui_dongle(cli_dongle, cfg.default_dongle().as_deref())
}
