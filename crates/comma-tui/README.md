# comma-tui

Lightweight terminal user interface for interacting with comma devices and the comma connect API.

The binary is named `comma` for convenience.

## Status

Early implementation. Tabbed shell (space-saving layout), fixed source-defined keybindings, shared config + env + CLI overrides, last-used dongle in state directory, logging, and placeholder data views are in place. Real API integration and richer tab content come next.

See `docs/` for design notes and decisions.

## Design goals (early versions)

- Text-only (no video, no thumbnails, no graphics protocols).
- Tabbed main screen inspired by rustnet.
- Fixed, source-defined keybindings (Vim-like, modifiers and multi-key support).
- Shared config with `comma-cli` (TOML via confy, app name `comma`). Respects the same environment variables (`COMMA_TOKEN`, `COMMA_DONGLE`). Last-used dongle is stored in the state directory.
- CLI flags override both.
- Dongle switcher + routes + upload queue as the first useful views.
- Shared helpers live in the `comma-common` crate.

## Building and running

```bash
cargo build -p comma-tui
./target/debug/comma --help
# With token (or set COMMA_TOKEN):
./target/debug/comma --token "$COMMA_TOKEN"
```

Config file location follows confy conventions (typically `~/.config/comma-tui/config.toml` on Linux).
