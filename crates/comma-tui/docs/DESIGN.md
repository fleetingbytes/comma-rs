# TUI Design Notes

These are working notes on the intended look, feel, and behaviour of the `comma` TUI.
They are not formal architecture decision records (ADRs); they capture use-case and
interaction design thinking so we can implement deliberately.

Related: see the individual notes in this directory.

## Core principles

- **Lightweight first.** Early versions are pure text. No video playback, no
  thumbnails, no half-block rendering, no graphics protocols.
- **Useful before pretty.** Prioritise routes list, upload queue, device overview,
  and dongle switching. Visual flourishes (coloured route markers, etc.) come later.
- **Fixed, source-defined keybindings.** Keybindings are not user-configurable.
  They live in one place in the source code (with associated display strings).
  Defaults are Vim-like, with support for modifiers and multiple keys per action.
- **Separate but compatible config.** The TUI has its own config file (TOML via
  confy, same strategy as `comma-cli`). Dongle ID and token can live there, but
  environment variables (`COMMA_TOKEN`, `COMMA_DONGLE`) are preferred and shared
  with `comma-cli`. CLI flags override both.

## Main screen layout (inspired by rustnet)

```
┌───────────────────────────────────────────────────────┐
│  Routes   Queue   Device    Devices   Help            │  ← tab bar
├───────────────────────────────────────────────────────┤
│                                                       │
│  (active tab content)                                 │
│                                                       │
│                                                       │
├───────────────────────────────────────────────────────┤
│  j/k: move   space: select   q: quit   ? : help       │  ← status / key hints
└───────────────────────────────────────────────────────┘
```

- Top line: list of tabs, active tab highlighted.
- Bottom line: context-sensitive keyboard shortcuts currently available
  (subset of bindings, ranked by importance, limited by terminal width).
- Rightmost tab is always **Help**, which lists all keybindings (including
  alternatives and modifiers) generated from the source-defined binding table.

## Tabs (initial set)

1. **Routes** — list of routes for the active dongle (metadata only).
2. **Queue** — upload queue for the active dongle.
3. **Device** — overview of the active dongle.
4. **Devices** — switch the active dongle. Lists all devices the user can access
   (owned + shared). Only one is active at a time; other tabs show data for it.
5. **Help** — full keybinding and command overview (generated from source).

## Dongle selection

- On startup, select the dongle from (in order): `--dongle` / `-d` CLI flag,
  `COMMA_DONGLE` environment variable, last-selected dongle stored in the TUI
  config, or prompt / first available.
- The Devices tab lets the user change the active dongle at runtime.
  The choice is persisted to the TUI config.
- All other data tabs operate on the currently selected dongle.
- Users who want to ignore a persisted or environment dongle temporarily can
  use `env -u COMMA_DONGLE comma` (or pass `--dongle`).

## Configuration

Own config file managed by `confy` (TOML, platform-standard location, same
strategy as `comma-cli`; typically `~/.config/comma-tui/config.toml` on Linux).

Example sketch:

```toml
default_dongle = "0123456789abcdef"
# token = "..."   # prefer env var
last_dongle = "0123456789abcdef"   # written by the TUI on dongle switch
```

Keybindings are **not** in the config; they are defined in source code.

## Keybindings

- Defined in a single place in the source code.
- Each binding has an associated display string (e.g. `space`, `enter`,
  `ctrl-f`) used by the bottom bar and Help tab. Changing a binding or its
  display string requires no other edits.
- Support for modifier keys (e.g. `Ctrl-F` / `Ctrl-B` for page up/down).
- Multiple keys may be bound to the same action (e.g. both `j`/`k` and arrow
  keys for movement).
- Help tab lists all bindings for each action.
- Bottom bar shows a ranked subset (by importance), truncated as needed for
  terminal width. The set of hints is context-sensitive per tab (Routes has
  upload actions, Queue has cancel, Device has alias editing, etc.).

## Feature parity with the CLI

The TUI should offer roughly the same interactive capabilities as the CLI
(routes listing, upload queue management, requesting uploads of cameras/logs/
routes/recent-routes, device alias changes, etc.).

- CLI remains the right tool for scripting, one-off commands, and raw JSON
  output.
- TUI translates API responses into human-friendly interactive views.
- Reusable helper logic currently living in `comma-cli/src/main.rs` (the
  various `cmd_*` functions) should be extracted into a shared module that
  both the CLI and TUI can use.

## Non-goals for early versions

- Rendering any frames from route video footage.
- Thumbnails or coloured route icons (possible later, e.g. time-of-day colour).
- Graphics protocol support (Kitty, Sixel, iTerm2) or half-block video.
- Replacing the web UI’s media viewing experience.
- User-configurable keybindings.

## Resolved questions

- **Config format and location:** TOML via `confy`, same strategy as
  `comma-cli`.
- **Bottom bar:** Fully dynamic and context-sensitive. Number of shown hints
  depends on terminal width and an importance ranking of the bindings.
- **Upload-request workflow:** Offer more or less the same features in CLI
  and TUI. Extract shared helpers from the CLI.
- **Persistence of last-selected dongle:** Written to the TUI config.
  Overridden by environment variable or CLI flag. Temporary override via
  `env -u COMMA_DONGLE comma`.
