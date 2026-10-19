# TDR-004: Fixed source-defined keybindings (not user-configurable)

## Context

Earlier design considered fully user-configurable keybindings (to accommodate
layouts such as Colemak-DH). After further thought, the complexity of a
configurable binding system (config parsing, validation, dynamic Help and
status-bar generation from user input, conflict handling) is not worth the
benefit for a focused TUI. A single source-of-truth in code is simpler,
easier to test, and still allows the display strings to stay in sync with
the actual keys.

## Decision

- Keybindings are **not** user-configurable.
- All bindings live in one place in the source code.
- Each binding carries its own display string (e.g. `space`, `enter`,
  `ctrl-f`, `up`). The bottom bar and Help tab are generated from this table,
  so changing a key or its label requires no other edits.
- Modifier keys are supported (examples: `Ctrl-F` / `Ctrl-B` for page
  up/down).
- Multiple keys may be bound to the same action (e.g. both `j`/`k` and the
  arrow keys for vertical movement).
- The Help tab lists every binding for each action.
- The bottom bar shows a ranked subset of the currently relevant bindings
  (importance rank + available terminal width). The set of hints is
  context-sensitive per tab.

## Consequences

- Much less implementation and testing complexity.
- Help and status bar stay automatically consistent with the code.
- Users who strongly prefer a different layout must either live with the
  defaults or fork/patch the binding table (acceptable trade-off).
- Easy to add alternative keys (arrows, etc.) without user configuration.

## Status

Accepted (2026-10-19) — supersedes the previous “fully configurable”
decision.
