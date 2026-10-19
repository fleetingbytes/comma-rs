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
- **Configurable interaction.** Keybindings must be fully user-definable because
  keyboard layouts differ (Colemak-DH, etc.). Defaults should feel natural to
  Vim users.
- **Separate but compatible config.** The TUI has its own config file. Dongle ID
  and token can live there, but environment variables (`COMMA_TOKEN`,
  `COMMA_DONGLE`) are preferred and shared with `comma-cli`. CLI flags override
  both.

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
- Bottom line: context-sensitive keyboard shortcuts currently available.
- Rightmost tab is always **Help**, which renders the current keybinding map
  (so it stays accurate after the user customises bindings).

## Tabs (initial set)

1. **Routes** — list of routes for the active dongle (metadata only).
2. **Queue** — upload queue for the active dongle.
3. **Device** — overview of the active dongle.
4. **Devices** — switch the active dongle. Lists all devices the user can access
   (owned + shared). Only one is active at a time; other tabs show data for it.
5. **Help** — dynamic keybinding and command overview.

## Dongle selection

- On startup, select the dongle from (in order): `--dongle` / `-d` CLI flag,
  `COMMA_DONGLE` environment variable, config file, or prompt / first available.
- The Dongles tab lets the user change the active dongle at runtime.
- All other data tabs operate on the currently selected dongle.

## Configuration

Own config file (e.g. `~/.config/comma-tui/config.toml` via confy or similar).

Example sketch:

```toml
default_dongle = "0123456789abcdef"
# token = "..."   # prefer env var

[keybindings]
# Vim-like defaults, overridable
up = "k"
down = "j"
left = "h"
right = "l"
select = "enter"
quit = "q"
help = "?"
# ...
```

Inspiration for the binding style can come from leftwm’s config, but we keep it
simple for v1.

## Keybindings

- Defaults are Vim-like (`j`/`k` for down/up, etc.).
- Completely configurable.
- The Help tab and the bottom status bar must reflect the user’s current bindings,
  not hard-coded strings.

## Non-goals for early versions

- Rendering any frames from route video footage.
- Thumbnails or coloured route icons (possible later, e.g. time-of-day colour).
- Graphics protocol support (Kitty, Sixel, iTerm2) or half-block video.
- Replacing the web UI’s media viewing experience.

## Open questions

- Exact config format and location.
- Whether the bottom bar is fully dynamic or has a fixed set of always-visible hints.
- How much of the upload-request workflow (road-camera, recent-routes, etc.)
  belongs in the TUI versus staying in the CLI for now.
- Persistence of the last-selected dongle across runs.
