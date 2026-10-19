# TDR-001: Lightweight text-only early versions

## Context

The comma web UI shows route video. Terminal video playback is possible via
graphics protocols (Kitty, iTerm2, Sixel) or unicode half-blocks, but:

- Stock Alacritty supports none of the graphics protocols.
- Half-block video is blocky and bandwidth-heavy.
- The primary value of a TUI for many users is metadata, queue management,
  and quick navigation, not media playback.

## Decision

Early versions of `comma` (the TUI) will contain **no** rendering of video
footage, not even still thumbnails or coloured route markers.

Routes are listed as plain text lines derived from metadata.

## Consequences

- Faster to implement and ship useful functionality.
- Works identically in Alacritty, over SSH, and in any truecolor terminal.
- Leaves the door open for optional visual enhancements later (time-of-day
  colour blocks, half-blocks, or protocol-based images) without blocking
  the core experience.

## Status

Accepted (2026-10-18)
