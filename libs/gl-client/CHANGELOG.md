# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Configure routing and on-chain fee limits with `FeeBudgetLimits` through `SignerConfig::fee_budget_limits`. See [signer fee budgets](../../docs/src/reference/signer-fees.md) for configuration, defaults, and limitations.

### Changed

- Enforce a routing-fee allowance per payment, shared across all parts and channels. The default is the larger of 1,000 sat and 0.5% of the payment amount. `pay`, direct-invoice `xpay`, and trampoline requests with fee limits above the allowance are rejected before HTLCs are offered.
- Enforce on-chain fee limits of 100,000 sat per new commitment or signed local HTLC input, and 50,000 sat per mutual close by default. Previously accepted force-close commitments remain signable after limits are lowered.

### Fixed

- Fix missing invoice approvals for invoice-backed `sendpay` and direct-invoice `xpay`, including amountless invoices.
- Allow retrying an amountless invoice with a different amount once the previous attempt has no outstanding HTLCs.
- Fix node hangs caused by signer errors during invoice and keysend preapproval.

## [0.7.0] - 2026-10-12

### Added

- `Scheduler::check_lightning_availability(node_id)` and the corresponding `Scheduler.CheckLightningAvailability` RPC — reports whether the Lightning account backed by `node_id` may be surfaced to the user. Unauthenticated and callable before the node is registered, so the `node_id` is passed explicitly rather than taken from the credentials. Monotonic: a registered node is always reported as available.
- Channel-splice support: a splice-policy dry-run resolver that answers splice decisions offline, durable splice state primitives in the `persist` module, and PSBT helpers (`psbt` module) for building splices from PSBTv2.

### Changed

- Signer version updated from `v25.12` to `v26.06` (VLS 0.14.0 / CLN v26.06.1)

### Fixed

- Signing operations no longer hang after a `signmessage` RPC call.

## [0.6.0] - 2026-05-21

### Added

- New `lnurl` module with full LNURL protocol support (LUD-01/03/06/09/10/16)
- `lnurl::pay::fetch_invoice()` free function for LNURL-pay invoice retrieval
- `lnurl::withdraw::build_withdraw_callback_url()` free function for LNURL-withdraw
- LNURL model types: `LnUrlPayRequestData`, `LnUrlWithdrawRequestData`, `LnUrlHttpClient` trait, `LnUrlHttpClearnetClient`
- Lightning Address parsing via `lnurl::pay::parse_lightning_address()`
- LNURL bech32 decoding via `lnurl::utils::parse_lnurl()`
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

- Use `webpki-roots` for LNURL TLS to support cross-platform builds (no dependency on system CA store)
- Hardened LNURL-pay against real-world service quirks
- Case-insensitive comparison for BOLT11 invoice strings in signer
- Addressed an issue with signers being unable to connect to the node, due to an SNI header override that is no longer required
- Parsing an invalid certificate no longer panics, instead returning an error
- Addressed a deprecation warning in gl-testing regarding `PROTOCOL_TLS` being renamed to `PROTOCOL_TLS_SERVER`
- Fixed initial VLS state not being persisted to the tower (nodelet)

[0.7.0]: https://github.com/Blockstream/greenlight/compare/gl-client-v0.6.0...gl-client-v0.7.0
[0.6.0]: https://github.com/Blockstream/greenlight/compare/gl-client-0.3.2...gl-client-v0.6.0
