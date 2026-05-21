# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## [0.4.0] - 2026-05-21

### Added

- `prepare_onchain_send()` to preview on-chain send fee, recipient amount, and UTXO selection without broadcasting
- `onchain_balance_state()` to classify wallet balance into UI-ready states (Available, ReserveOnly, PendingConfirmation, Immature, Unavailable)
- `onchain_fee_rates()` to retrieve fee rate buckets (next-block through 1-day) from the node
- New types: `PreparedOnchainSend`, `Outpoint`, `OnchainFeeRates`, `OnchainBalanceState`
- C++ bindings support via UniFFI with build tasks for generating headers and shared library
- `parse_input()` — synchronous, offline classification of user input. Recognises BOLT11 invoices, node IDs, LNURL bech32 strings (decoded to their underlying URL), and Lightning Addresses (returned as the unparsed `user@host` form). LNURL inputs are classified but not fetched; the cost contract is "no HTTP, no I/O."
- `resolve_input()` — network-touching classification that resolves LNURL/Lightning Address inputs via HTTP, returning typed pay or withdraw request data. BOLT11 invoices and node IDs pass through without I/O.
- `ParsedInput` enum (offline result): `Bolt11`, `NodeId`, `LnUrl { url }`, `LnUrlAddress { address }`
- `ResolvedInput` enum (resolved result): `Bolt11`, `NodeId`, `LnUrlPay { data }`, `LnUrlWithdraw { data }`
- Builder-style `Node` creation with signerless mode as default
- `NodeState` snapshot with hex identifiers
- `LogListener` for foreign-binding log capture
- `generate_diagnostic_data()` for support dumps
- `Node::get_info()` method for retrieving node identity and status
- `Node::list_peers()` method for listing connected peers
- `Node::list_peer_channels()` method for detailed channel information
- `Node::list_funds()` method for on-chain and channel fund balances
- `Node::stream_node_events()` for real-time event streaming (invoice payments, etc.)
- `Node::list_invoices()`, `Node::list_pays()`, and `Node::list_payments(request)` for payment history with request-based filtering
- `DeveloperCert` type and `Scheduler::with_developer_cert()` builder for runtime certificate injection
- `Signer::new_from_seed()` constructor as an alternative to raw secret bytes
- Top-level `register()`, `recover()`, `connect()`, and `register_or_recover()` convenience functions for simplified onboarding
- `Config` type for SDK configuration (network, developer cert)
- `opening_fee_msat` field to `ReceiveResponse` reporting LSP JIT channel fees
- Many new exported types: `GetInfoResponse`, `ListPeersResponse`, `ListPeerChannelsResponse`, `ListFundsResponse`, `ListInvoicesResponse`, `ListPaysResponse`, `ListPaymentsRequest`, `Invoice`, `InvoiceStatus`, `Pay`, `Payment`, `PaymentType`, `PaymentTypeFilter`, `PaymentStatus`, `ListIndex`, `Peer`, `PeerChannel`, `FundChannel`, `FundOutput`, `NodeEvent`, `NodeEventStream`, `InvoicePaidEvent`, `ChannelState`, `OutputStatus`

### Changed

- `onchain_send()` now accepts optional `sat_per_vbyte` and `utxos` parameters for fee-rate control and UTXO pinning
- Error variants now carry stable numeric `code`, human-readable `message`, and structured `values` map for i18n interpolation — replaces tuple-style error variants
- `resolve_input()` is now **synchronous** — the entire public SDK API exposed through UniFFI is blocking, with async work handled internally on a shared Tokio runtime
- Removed the `cpp-bindings` Cargo feature flag — a single build of the shared library now works for all language bindings (Python, Kotlin, Swift, Ruby, C++) without conditional compilation
- Response types migrated from `uniffi::Object` to `uniffi::Record` so struct fields are directly accessible from bindings
- Made `Credentials::load()`, `Credentials::save()`, `Node::receive()`, `Node::send()`, `Node::onchain_send()`, `Node::onchain_receive()`, `Signer::new()`, `Signer::authenticate()`, `Signer::start()`, `Signer::node_id()` public

### Removed

- `Node::resolve_lnurl()` and the `ResolvedLnUrl` enum. Use `parse_input()` (offline) or `resolve_input()` (HTTP) to obtain `LnUrlPayRequestData` / `LnUrlWithdrawRequestData`, then call `Node::lnurl_pay()` / `Node::lnurl_withdraw()`.

### Fixed

- Hardened LNURL-pay against real-world services
- Excluded unpaid invoices from `list_payments`
- `list_payments` fixes for edge cases
- Fixed msat parsing in `onchain_send` `amount_or_all` parameter

## [0.1.1] - 2026-01-16

### Changed

- Updated gl-client dependency to support CLN v25.12 signer.

[0.4.0]: https://github.com/Blockstream/greenlight/compare/gl-sdk-v0.1.1...gl-sdk-v0.4.0
[0.1.1]: https://github.com/Blockstream/greenlight/releases/tag/gl-sdk-v0.1.1
