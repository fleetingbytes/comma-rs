# TDR-002: Tabbed layout inspired by rustnet

## Context

We want a clear, keyboard-driven main screen. Two reference TUIs are rustnet
(ratatui, tabbed) and btop (more dashboard-like). The preference is closer to
rustnet.

## Decision

The main screen is tabbed:

- Top bar lists tabs and highlights the active one.
- Bottom bar shows currently available keybindings.
- Rightmost tab is always Help and is generated from the live keybinding config.

Initial tabs: Routes, Queue, Devices, Dongles, Help.

## Consequences

- Familiar mental model for users of rustnet-style tools.
- Help stays accurate when keybindings are customised.
- Easy to add further tabs later without redesigning the shell.

## Status

Accepted (2026-10-18)
