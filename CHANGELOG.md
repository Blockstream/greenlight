# Changelog

All notable changes to the subprojects will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## Unreleased

-

## [0.3.0] - 2024-10-07

### Added

- Capture and report signer rejections back to the node ([#483](https://github.com/Blockstream/greenlight/pull/483), #[484](https://github.com/Blockstream/greenlight/pull/484)).
- Add trampoline client to delegate route finding to next node ([#475](https://github.com/Blockstream/greenlight/pull/475), #[489](https://github.com/Blockstream/greenlight/pull/489), #[498](https://github.com/Blockstream/greenlight/pull/498), #[505](https://github.com/Blockstream/greenlight/pull/505), #[511](https://github.com/Blockstream/greenlight/pull/511)).
- Add basic support for signer-less devices ([#281](https://github.com/Blockstream/greenlight/pull/281)).

### Changed

- Ensure that signer doesn't exit on network change ([#524](https://github.com/Blockstream/greenlight/pull/524))
- Signer reports the node id by itself ([#520](https://github.com/Blockstream/greenlight/pull/520))
- Upgrade to VLS 0.12 ([#504](https://github.com/Blockstream/greenlight/pull/504)).
- Add .resources dir to all crates in repo. This is soley to make it possible to publish artefacts ([#501](https://github.com/Blockstream/greenlight/pull/501))

### Fixed

- Several small drive-by fixes ([#468](https://github.com/Blockstream/greenlight/pull/468), [#471](https://github.com/Blockstream/greenlight/pull/471), [#508](https://github.com/Blockstream/greenlight/pull/508))
