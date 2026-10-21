# TDR-003: Shared config with environment variables; last-used dongle in state

## Context

Earlier design used separate confy configs for the CLI (`comma-cli`) and the TUI (`comma-tui`).
After review we prefer a single shared configuration file, with TUI-specific settings in their own table.
Last-used dongle is application state, not configuration, so it belongs in the state directory.

## Decision

- CLI and TUI share one config file, managed by confy with app name `"comma"`
  (typically `~/.config/comma/config.toml` on Linux).
- Structure:

  ```toml
  [common]
  default_dongle_id = "0123456789abcdef"
  # token = "..."          # prefer env var

  [tui]
  # TUI-specific settings (future)
  ```

- Both tools honour `COMMA_TOKEN` and `COMMA_DONGLE`.
- CLI flags (`--dongle`, `--token`) override both config and environment.
- Last-used dongle for the TUI is stored in the state directory
  (`$XDG_STATE_HOME/comma` or `~/.local/state/comma`) as a plain file
  `tui-last-used-device-id`. It is **not** stored in the config.
- Startup dongle selection order for the TUI:
  1. CLI `--dongle` / `-d`
  2. `COMMA_DONGLE` environment variable
  3. last-used state file
  4. `common.default_dongle_id` from config
- If no dongle can be resolved, the TUI starts on the Devices tab.
- If no token is available (or later, if the token cannot list devices),
  the TUI does not start; it writes an error to stderr explaining how to
  obtain a JWT at https://useradmin.comma.ai (click "Get JWT").
- Keybindings are not stored in the config; they are defined in source
  (see TDR-004).
- Logs live under the same state directory:
  - `comma-cli.log`
  - `comma-tui.log`

## Consequences

- Users can keep secrets out of config files via environment variables.
- Shared settings do not have to be duplicated.
- Last-used dongle is treated as state, not configuration.
- Temporary override of a persisted or environment dongle is possible with
  `env -u COMMA_DONGLE comma` or an explicit `--dongle` flag.

## Status

Accepted (2026-10-21) — supersedes the previous separate-config decision.
