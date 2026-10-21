# TDR-002: Tabbed layout inspired by rustnet

## Context

We want a clear, keyboard-driven main screen. Two reference TUIs are rustnet
(ratatui, tabbed) and btop (more dashboard-like). The preference is closer to
rustnet.

## Decision

The main screen is tabbed:

- Top bar: left-aligned tabs (active highlighted), right-aligned device
  status indicator + alias/ID. No surrounding frame.
- Content area has no frame; it starts immediately under the separator line.
- Bottom bar: left-aligned ranked, context-sensitive hints that fit in the
  remaining width; right-aligned always-present `? help  q quit`.
  Generated from the source-defined binding table.
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
