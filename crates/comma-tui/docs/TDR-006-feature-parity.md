# TDR-006: Feature parity with CLI and shared helpers

## Context

The CLI already implements a useful set of operations (route listing, upload
queue inspection and cancellation, requesting uploads of individual file
kinds, whole routes, and recent routes, device information, etc.). The TUI
should not reinvent these; it should expose roughly the same capabilities in
an interactive form.

## Decision

- The TUI will offer more or less the same interactive features as the CLI.
- The CLI remains the appropriate tool for scripting, one-off commands, and
  obtaining raw JSON API responses.
- The TUI translates those responses into human-friendly interactive views.
- Reusable logic currently living in `comma-cli/src/main.rs` (the various
  `cmd_*` helper functions) should be extracted into a shared module that
  both the CLI and the TUI can depend on.
- As features are added, continuously check whether new helpers belong in
  the shared module rather than being duplicated.

## Consequences

- Avoids divergent behaviour between CLI and TUI.
- Keeps the TUI focused on presentation and interaction while reusing
  proven request/response handling.
- Slightly more up-front refactoring of the CLI, but pays off as the TUI
  grows.

## Status

Accepted (2026-10-19)
