# comma-tui

Lightweight terminal user interface for interacting with comma devices and the comma connect API.

The binary is named `comma` for convenience.

## Status

Early design phase. No real UI yet. See `docs/` for TUI design notes and decisions.

## Design goals

- Text-only early versions (no video, no thumbnails, no graphics protocols).
- Tabbed main screen inspired by rustnet.
- Fully configurable keybindings (Vim-like defaults).
- Separate config from `comma-cli`, but respects the same environment variables.
- Dongle switcher + routes + upload queue as the first useful views.

## Building

```bash
cargo build -p comma-tui
./target/debug/comma
```
