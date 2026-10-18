# TDR-003: Separate config with shared environment variables

## Context

`comma-cli` already stores a default dongle ID and token via confy
(`~/.config/comma-cli/config.toml`) and prefers `COMMA_TOKEN` /
`COMMA_DONGLE` environment variables.

The TUI has different needs (keybindings, possibly more UI settings) and
different usage patterns.

## Decision

- The TUI has its **own** config file (separate from `comma-cli`).
- It may store a default dongle ID and token.
- It **must** honour the same environment variables (`COMMA_TOKEN`,
  `COMMA_DONGLE`) that the CLI uses.
- CLI flags (`--dongle`, `--token`) override both config and environment.
- Keybindings live only in the TUI config.

Duplication of dongle/token is acceptable; environment variables remain the
preferred shared mechanism.

## Consequences

- Users can keep secrets out of config files via environment variables.
- TUI-specific settings (keybindings) do not pollute the CLI config.
- Startup selection order is well-defined: CLI flag > env > config > fallback.

## Status

Accepted (2026-10-18)
