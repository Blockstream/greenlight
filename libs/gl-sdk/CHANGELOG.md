# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

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
