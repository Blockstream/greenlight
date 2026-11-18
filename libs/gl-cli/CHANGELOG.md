# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.2.0] - 2026-11-18

### Added

- `node init` creates the local data directory and the `hsm_secret`, from a 12-word mnemonic you provide or a newly generated one, which is shown on screen.
- `node newaddr`, `node listfunds`, `node withdraw`, `node fundchannel` and `node close` commands.
- `--json` flag for machine-readable JSON output on all commands (invoice, pay, listpays, connect, stop, close, fundchannel, withdraw, listfunds, newaddr)
- `signer run` takes `--state-signature-mode` (`off`, `soft` or `hard`), `--state-override` and `--state-override-note`.
- `signer run` writes a local signer backup with `--backup-path`, `--backup-strategy` and `--backup-periodic-updates`. `signer inspect-backup` reads a backup, and `signer convert-backup` converts it to CLN's channel recovery format.
- `signer run` takes the signer's fee budgets: `--payment-fee-exempt-msat`, `--max-payment-fee-ppm`, `--max-commitment-fee-sat` and `--max-mutual-close-fee-sat`.

### Changed

- `fundchannel` now creates private channels by default
- Updated `gl-client` to 0.8.0.

### Removed

- Seeds not derived from a BIP39 mnemonic. Every seed now comes from a 12-word mnemonic, through `node init` or `scheduler register`, so every wallet can be recovered from its mnemonic.

## [0.1.2] - 2026-01-16

### Added

- Added `scheduler export` subcommand to take control of your own node.

### Changed

- Updated gl-client dependency to support CLN v25.12 signer.

[0.2.0]: https://github.com/Blockstream/greenlight/releases/tag/gl-cli-v0.2.0
[0.1.2]: https://github.com/Blockstream/greenlight/commit/72c877a98fea83f671824c41251417d131ef9086
