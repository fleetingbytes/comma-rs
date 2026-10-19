# TDR-002: Tabbed layout inspired by rustnet

## Context

We want a clear, keyboard-driven main screen. Two reference TUIs are rustnet
(ratatui, tabbed) and btop (more dashboard-like). The preference is closer to
rustnet.

## Decision

The main screen is tabbed:

- Top bar lists tabs and highlights the active one.
- Bottom bar shows currently relevant keybindings (ranked subset, limited by
  terminal width; generated from the source-defined binding table).
- Rightmost tab is always Help and lists all bindings (including alternatives
  and modifiers) generated from the same source table.

Initial tabs: Routes, Queue, Device, Devices, Help.

## Consequences

- Familiar mental model for users of rustnet-style tools.
- Help and bottom bar stay automatically consistent with the code-defined
  bindings.
- Easy to add further tabs later without redesigning the shell.
- Bottom bar can be context-sensitive per tab without extra configuration.

## Status

Accepted (2026-10-19)
