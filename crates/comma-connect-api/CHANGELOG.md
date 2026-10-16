# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/fleetingbytes/comma-rs/releases/tag/comma-connect-api-v0.1.0) - 2026-10-16

### Other

- set up independent semantic versioning with release-plz and CI workflows
- Add upload queue cancellation commands
- Add upload recent-routes
- Add upload-road-camera command
- Add upload-queue command via Athena listUploadQueue
- Fix route file status and add cucumber tests
- Add route-segments command and Segment type
- Fix Route deserialization: accept ISO timestamps as strings for start_time/end_time
- Add get_routes method and Route struct for listing uploaded routes
- Add tracing logs to API client requests
- Add tracing dependency to comma-connect-api
- Fix Profile deserialization to tolerate null fields from /v1/me/
- Initialize crates: comma-connect-api client and comma-cli with config + basic API calls
