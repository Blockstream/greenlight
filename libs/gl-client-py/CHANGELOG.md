# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Optional `state_signature_mode` and `device_storage_path` keyword arguments for `Signer`, allowing explicit one-time Soft migration with persistent storage. Existing constructors continue to use Hard by default.

## [0.4.0] - 2026-10-12

### Changed

- Signer version updated from `v25.12` to `v26.06` (VLS 0.14.0 / CLN v26.06.1)
- Remove the use of `decodepay`, deprecated in v24.11.
