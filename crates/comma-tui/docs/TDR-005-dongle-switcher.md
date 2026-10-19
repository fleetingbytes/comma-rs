# TDR-005: Dongle switcher tab (Devices)

## Context

A user may own multiple devices or have been granted read access to devices
owned by others (via useradmin.comma.ai). The TUI needs a clear way to choose
which dongle’s data is shown in the other tabs.

## Decision

- There is a dedicated **Devices** tab that lists all accessible devices.
- Exactly one dongle is active at a time.
- Other tabs (Routes, Queue, Device, …) always operate on the active dongle.
- On startup the active dongle is chosen by: CLI `--dongle`/`-d` >
  `COMMA_DONGLE` env > last-selected dongle stored in the TUI config >
  first available (or prompt).
- Changing the selection in the Devices tab updates the active dongle for the
  rest of the session **and** persists the choice to the TUI config.
- Users can temporarily ignore the persisted or environment value with
  `env -u COMMA_DONGLE comma` or by passing `--dongle`.

## Consequences

- Clear model for multi-device users.
- CLI and environment overrides are honoured, matching `comma-cli` behaviour.
- Avoids the need to restart the TUI when switching devices.
- Last-selected dongle survives across runs without requiring the user to
  edit the config by hand.

## Status

Accepted (2026-10-19)
