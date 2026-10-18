# TDR-004: Fully configurable keybindings with Vim-like defaults

## Context

Users have different keyboard layouts (e.g. Colemak-DH-ISO). Hard-coded
bindings would be painful. At the same time, Vim-style navigation is a
reasonable default for a terminal tool.

## Decision

- All interactive keybindings are configurable via the TUI’s config file.
- Defaults are Vim-like (`j`/`k` down/up, `h`/`l` left/right, etc.).
- The Help tab and the bottom status bar render the **current** bindings,
  not hard-coded strings.
- Style of the binding table can take light inspiration from leftwm’s config,
  but v1 can stay simple (string-to-action map).

## Consequences

- Works for non-QWERTY users without forking.
- Help and hints stay truthful.
- Slightly more implementation work up front (keybinding system + dynamic UI).

## Status

Accepted (2026-10-18)
