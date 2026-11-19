# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Wheels are published to PyPI from GitHub Actions again, for Linux (x86_64, i686), macOS (x86_64, arm64) and Windows (x64, x86). 0.3.4 had shipped only a Linux x86_64 wheel.

## [0.4.1] - 2026-11-18

### Changed

- Updated `gl-client` to 0.8.0. The signer enforces per-payment routing-fee and on-chain fee budgets with their default limits, and uses VLS 1.0.0-rc.1.

## [0.4.0] - 2026-10-12

### Changed

- Signer version updated from `v25.12` to `v26.06` (VLS 0.14.0 / CLN v26.06.1)
- Remove the use of `decodepay`, deprecated in v24.11.

[0.4.1]: https://github.com/Blockstream/greenlight/releases/tag/gl-client-py-v0.4.1
[0.4.0]: https://github.com/Blockstream/greenlight/commit/97db7225e15ec13e6a917552f3f489b41ca4424c
