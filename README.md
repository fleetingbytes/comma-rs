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
   ./target/release/comma-cli config show
   ./target/release/comma-cli config set dongle 0123456789abcdef
   ./target/release/comma-cli me
   ./target/release/comma-cli devices
   ./target/release/comma-cli upload queue list
   ./target/release/comma-cli upload recent-routes
   ```

The config file (TOML) is stored in the platform-standard location managed by `confy` (usually `~/.config/comma-cli/config.toml` on Linux).

## Current status

The CLI and API client support:

- Configuration (default dongle, token)
- Profile, devices, routes, route files
- Athena upload queue listing and cancellation (`upload queue list`, `upload queue cancel id`, `upload queue cancel pending`)
- Requesting uploads of individual file kinds, whole routes, and recent routes

Further features will be added incrementally.
