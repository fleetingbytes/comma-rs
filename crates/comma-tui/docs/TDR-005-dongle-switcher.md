# TDR-005: Dongle switcher tab

## Context

A user may own multiple devices or have been granted read access to devices
owned by others (via useradmin.comma.ai). The TUI needs a clear way to choose
which dongle’s data is shown in the other tabs.

## Decision

- There is a dedicated **Dongles** tab that lists all accessible devices.
- Exactly one dongle is active at a time.
- Other tabs (Routes, Queue, Devices, …) always operate on the active dongle.
- On startup the active dongle is chosen by: CLI `--dongle`/`-d` >
  `COMMA_DONGLE` env > config > first available (or prompt).
- Changing the selection in the Dongles tab updates the active dongle for the
  rest of the session (persistence across runs is optional for later).

## Consequences

- Clear model for multi-device users.
- CLI override is honoured, matching `comma-cli` behaviour.
- Avoids the need to restart the TUI when switching devices.

## Status

Accepted (2026-10-18)
