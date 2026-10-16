# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/fleetingbytes/comma-rs/compare/comma-cli-v0.1.0...comma-cli-v0.1.1) - 2026-10-16

### Other

- set up independent semantic versioning with release-plz and CI workflows
- Add upload queue cancellation commands
- Add upload recent-routes
- Add upload route for all file kinds of one route
- Rename upload logs to upload log
- Add upload logs and generalize FileKind
- Add wide-road-camera and driver-camera uploads
- Nest config under subcommands
- Nest route and upload under subcommands
- Add upload-road-camera command
- Add upload-queue command via Athena listUploadQueue
- Fix route file status and add cucumber tests
- Add route-segments command and Segment type
- Add concurrent file logging to ~/.local/state/comma-rs/comma-cli.log
- Depend on tracing-appender in comma-cli
- Add routes subcommand to list uploaded routes for a device
- Add -v/--verbose flag and tracing subscriber initialization
- Add tracing and tracing-subscriber to comma-cli
- Initialize crates: comma-connect-api client and comma-cli with config + basic API calls
