# comma-cli

Command-line tool for interacting with comma devices through the comma connect API.

## Install

`cargo install comma-cli`

## Quick start

1. Obtain a developer JWT token from <https://jwt.comma.ai>.

2. Either export it:

   ```bash
   export COMMA_TOKEN=eyJ...
   ```

   or store it in the config (less preferred):

   ```bash
   comma-cli config set token <your token>
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
