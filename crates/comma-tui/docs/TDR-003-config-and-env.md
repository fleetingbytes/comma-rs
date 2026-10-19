# TDR-003: Separate config with shared environment variables

## Context

`comma-cli` already stores a default dongle ID and token via confy
(`~/.config/comma-cli/config.toml`) and prefers `COMMA_TOKEN` /
`COMMA_DONGLE` environment variables.

The TUI has different needs (persistence of last-selected dongle, possibly
other UI state) and different usage patterns. Keybindings are deliberately
not part of the config.

## Decision

- The TUI has its **own** config file, managed by `confy` (TOML, same
  strategy and platform-standard locations as `comma-cli`).
- It may store a default dongle ID, token, and the last-selected dongle.
- It **must** honour the same environment variables (`COMMA_TOKEN`,
  `COMMA_DONGLE`) that the CLI uses.
- CLI flags (`--dongle`, `--token`) override both config and environment.
- Keybindings are **not** stored in the config; they are defined in source
  code (see TDR-004).

Duplication of dongle/token is acceptable; environment variables remain the
preferred shared mechanism.

## Consequences

- Users can keep secrets out of config files via environment variables.
- TUI-specific state (last dongle) does not pollute the CLI config.
- Startup selection order is well-defined: CLI flag > env > config (including
  last-selected) > fallback.
- Temporary override of a persisted or environment dongle is possible with
  `env -u COMMA_DONGLE comma` or an explicit `--dongle` flag.

## Status

Accepted (2026-10-19)
