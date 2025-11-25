# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.1.2 - 2025-11-25

### Fixed

- Updated the hashes for v24.02gl1 and v24.11gl1. We applied some server-side patches for these versions.

### Changed

- Adjusted the expected hashes for artifacts, after patching them
- Inverted responsibilities: the CI now maintains the list of versions, along with authentication and integrity data, while the clnvm binary just uses the index.
