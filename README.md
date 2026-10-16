# comma-rs

Command-line tools and libraries for interacting with comma devices and the comma connect API.

## Crates

- `comma-connect-api`: Async Rust client for `https://api.commadotai.com`
- `comma-cli`: CLI application that demonstrates configuration handling and basic API calls
- `comma-test-utils`: Shared helpers for cucumber/BDD tests (not published)

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

## Contributing

We use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. This enables automatic semantic versioning and changelog generation via [release-plz](https://release-plz.dev/).

Examples:

- `feat(cli): add support for ...` (minor bump for the CLI crate)
- `fix(api): handle null email` (patch bump for the API crate)
- `feat!: breaking change to Client API` (major bump)
- `chore: update dependencies`

Scopes (e.g. `(cli)`, `(api)`) are optional but helpful. release-plz determines which crates to version based on which package files changed in the commits.

### CI/CD

- Pushes and PRs to `master` run CI (fmt, clippy, tests excluding live API integration tests, build).
- On push to `master`, release-plz creates/updates a release PR with version bumps and changelogs for affected crates, and publishes unpublished crates to crates.io.

Set the `CARGO_REGISTRY_TOKEN` repository secret (with `publish-new` and `publish-update` scopes) for publishing. For first-time publishes of new crates, you may need to publish manually once before trusted publishing can be fully used.

See the [release-plz quickstart](https://release-plz.dev/docs/github/quickstart) for more details, including enabling workflow permissions to create PRs.
