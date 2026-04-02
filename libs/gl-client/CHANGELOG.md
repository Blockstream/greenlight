# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0] - 2026-04-02

### Added

- New `metrics` module for signer state transfer instrumentation
- New `Error::IllegalArgument` variant for improved error reporting
- Signer now reports rejections to the server for debugging
- Retry logic for `get_pairing_data` to improve reliability
- VLS state synchronization with tombstone and conflict detection support
- Initial VLS state is now correctly synced to the nodelet on first connect
- Signer state canonicalization for deterministic serialization
- Signing modes and state override mode for signer state management
- Signature persistence in signer state
- Policy-other warn filter for commitment number desync (avoids noisy warnings)

### Changed

- Signer version updated from `v25.05` to `v25.12` (VLS 0.14.0 / CLN v25.12)
- Signer error handling: replaced panics with proper error propagation in `handler()`, `initmsg()`, and init message parsing
- State merge now detects and logs conflicts instead of silently overwriting

### Removed

- **BREAKING**: Removed the `lsps` module (`gl_client::lsps`). LSP functionality is now handled server-side via the plugin's `lsp_invoice` RPC.

### Fixed

- Addressed an issue with signers being unable to connect to the node, due to an SNI header override that is no longer required
- Parsing an invalid certificate no longer panics, instead returning an error
- Addressed a deprecation warning in gl-testing regarding `PROTOCOL_TLS` being renamed to `PROTOCOL_TLS_SERVER`
- Fixed initial VLS state not being persisted to the tower (nodelet)
