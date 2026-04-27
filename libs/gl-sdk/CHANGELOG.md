# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `Node::lnurl_auth(request)` implementing LNURL-auth (LUD-04 / LUD-05). Derives the per-domain linking key on-the-fly from a hardened `m/138'` xpriv that the Node derives once from the BIP39 seed at register/recover/connect time. The seed itself is never retained on `Node`; the stored namespace xpriv lives in `Zeroizing` and is scrubbed on `disconnect()` or `Drop`. `m/138'` is hardened, so the stored material cannot be used to derive any other wallet key (lightning channels, on-chain funds) — the blast radius is restricted to LNURL-auth identities.
- LUD-05 derivation uses the 32-byte private key at `m/138'/0` as the HMAC key, matching the mainstream wallet convention (Phoenix, Mutiny, Zeus, BlueWallet) for cross-wallet identity portability at LNURL-auth services.
- `InputType::LnUrlAuth { data: LnUrlAuthRequestData }` returned by `parse_input` when the URL carries `tag=login`. Detection is offline — no HTTP fetch is made for classification.
- `LnUrlCallbackStatus` and `LnUrlAuthRequestData` types.

### Changed

- `parse_input()` is now `async` and resolves LNURL bech32 strings and Lightning Addresses end-to-end over HTTP, returning typed pay or withdraw request data. BOLT11 invoices and node IDs still resolve without I/O.
- `InputType` variants now: `Bolt11`, `NodeId`, `LnUrlPay`, `LnUrlWithdraw`, `LnUrlAuth`. Replaces the previous `LnUrl` / `LnUrlAddress` intermediate-state variants.

### Removed

- `Node::resolve_lnurl()` and the `ResolvedLnUrl` enum. Use `parse_input()` to obtain `LnUrlPayRequestData` / `LnUrlWithdrawRequestData` directly, then call `Node::lnurl_pay()` / `Node::lnurl_withdraw()`.

## [0.2.0] - 2026-04-02

### Added

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
- `parse_input()` function for parsing BOLT11 invoices and node IDs
- `opening_fee_msat` field to `ReceiveResponse` reporting LSP JIT channel fees
- Many new exported types: `GetInfoResponse`, `ListPeersResponse`, `ListPeerChannelsResponse`, `ListFundsResponse`, `ListInvoicesResponse`, `ListPaysResponse`, `ListPaymentsRequest`, `Invoice`, `InvoiceStatus`, `Pay`, `Payment`, `PaymentType`, `PaymentTypeFilter`, `PaymentStatus`, `ListIndex`, `Peer`, `PeerChannel`, `FundChannel`, `FundOutput`, `NodeEvent`, `NodeEventStream`, `InvoicePaidEvent`, `ChannelState`, `OutputStatus`

### Fixed

- Fixed msat parsing in `onchain_send` `amount_or_all` parameter

### Changed

- Response types migrated from `uniffi::Object` to `uniffi::Record` so struct fields are directly accessible from bindings
- Made `Credentials::load()`, `Credentials::save()`, `Node::receive()`, `Node::send()`, `Node::onchain_send()`, `Node::onchain_receive()`, `Signer::new()`, `Signer::authenticate()`, `Signer::start()`, `Signer::node_id()` public

## [0.1.1] - 2026-01-16

### Changed

- Updated gl-client dependency to support CLN v25.12 signer.

[0.2.0]: https://github.com/Blockstream/greenlight/compare/gl-sdk-v0.1.1...gl-sdk-v0.2.0
[0.1.1]: https://github.com/Blockstream/greenlight/releases/tag/gl-sdk-v0.1.1
