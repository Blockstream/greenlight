# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0] - 2026-11-18

### Changed

- The proxy between CLN's `hsmd` connections and the signer runs fully asynchronously on tokio, with each subdaemon connection handled by its own task instead of a thread.
- Updated `tonic` to 0.11 and `prost` to 0.12.
- Synced the bundled protocol definitions with Greenlight.

[0.4.0]: https://github.com/Blockstream/greenlight/releases/tag/gl-signerproxy-v0.4.0
