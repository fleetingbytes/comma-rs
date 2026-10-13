# comma-rs

Command-line tools and libraries for interacting with comma devices and the comma connect API.

## Crates

- `comma-connect-api`: Async Rust client for `https://api.commadotai.com`
- `comma-cli`: CLI application that demonstrates configuration handling and basic API calls

## Quick start

1. Obtain a developer JWT token from <https://jwt.comma.ai>.

2. Either export it:

   ```bash
   export COMMA_TOKEN=eyJ...
   ```

   or store it in the config (less preferred):

   ```bash
   # After building, you can edit the config file shown by `comma-cli config`
   ```

3. Build and run:

   ```bash
   cargo build --release
   ./target/release/comma-cli config
   ./target/release/comma-cli set-dongle 0123456789abcdef
   ./target/release/comma-cli me
   ./target/release/comma-cli devices
   ./target/release/comma-cli device
   ```

The config file (TOML) is stored in the platform-standard location managed by `confy` (usually `~/.config/comma-cli/config.toml` on Linux).

## Current status

This is an early skeleton focused on validating:

- Configuration loading/storing (default dongle ID)
- Token resolution from environment variable or config
- Meaningful responses from the live comma API (`/v1/me/`, `/v1/me/devices/`, `/v1.1/devices/:id/`)

Further crates and features will be added incrementally.
