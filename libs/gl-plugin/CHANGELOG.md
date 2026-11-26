# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Signer presence on the `Stage`: a `watch` of `Presence::Answering` or `Presence::Silent` that can go back to `Silent`. A signer is answering after its first reply following an attach, and silent once its stream exits or a request stays unanswered for 10 s. `Stage::check_answering` gates dials: a reply within 10 s answers at once, otherwise it sends a `GetHeartbeat` probe and waits up to 5 s. `Stage::await_answering` waits for a signer within a deadline.

### Changed

- An unanswered ECDH request now marks the node as stuck, since it freezes all of `connectd`.
- The signer request stream ends as soon as the signer drops it, instead of on the next request.

## [0.4.0] - 2026-11-18

### Added

- `StreamNodeEvents` gRPC method, streaming node events as `NodeEvent` messages. The only event so far is `InvoicePaid`.
- The `lsp_invoice` response includes `opening_fee_msat`, the fee the LSP charges for opening the JIT channel, so clients can show it with the invoice.
- `lsp_invoice` stores metadata about JIT channel requests in the datastore, keyed by invoice label.
- A public helper looking up the configured `close_to` address, for downstream plugins that register their own `openchannel` hook.
- Support for CLN's `lsps-lsps2-invoice` RPC, the new name of `lsps-invoice`.

### Changed

- `lsp_invoice` creates a regular invoice instead of negotiating a JIT channel when the node already has enough incoming capacity, with a 5% buffer for fees.
- LSPs are picked in `node_id` order instead of at random, and LSPs that return no opening fee parameters are skipped.
- Signer state is synced incrementally, with a full re-sync when a conflict is detected.
- A channel is used for sends, invoices, route hints and channel waits only when it is in `CHANNELD_NORMAL` and either has a short channel id or has held funds for us. Incoming payments are no longer routed into JIT channels the LSP may have abandoned.
- New nodes get a longer startup timeout, avoiding spurious `No such file or directory` errors while the node initializes its database.
- A node blocked on onchaind signing requests, `GET_PER_COMMITMENT_POINT` or `FORGET_CHANNEL` reports itself as stuck, so the signer is summoned instead of the session idling until its time limit.
- Updated `gl-client` to 0.8.0.

### Fixed

- Use checked fee arithmetic for trampoline payments, rejecting invalid percentages and amount overflow.
- Preapprove invoice-backed `sendpay` calls before offering HTLCs, while the signed request context is still available to the signer.
- Waiting for a channel fails as soon as the channel is closed or abandoned, instead of waiting until the deadline.
- Invoice creation no longer hangs while peers are reconnected after a signer attaches. Peers are dialed in parallel on their own RPC connections with a 30s timeout, instead of one after the other on the connection `lsp_invoice` also uses, where a single unreachable peer stalled it for minutes.
- `lsp_invoice` uses its own RPC connection and gives up after 30s, so an LSP that never answers no longer hangs the invoice and every other RPC call behind it.
- A `signmessage` call no longer blocks all further signing. The signer refuses `SIGN_MESSAGE`, so the RPC is rejected with `Unimplemented` instead of being queued ([#739](https://github.com/Blockstream/greenlight/issues/739)).

### Removed

- Legacy client-side LSPS2 `htlc_accepted` hook — JIT channel fee handling is now performed natively by CLN
- Remove `Decodepay` rpc, deprecated in v24.11.
- Compatibility code for nodes older than v25.12gl1.

## [0.3.1] - 2026-01-16

### Changed

- Added message type 28 (`hsmd_check_pubkey`) to the list of sticky signer request.
- Updated gl-client dependency to support CLN v25.12 signer.

[0.4.0]: https://github.com/Blockstream/greenlight/releases/tag/gl-plugin-v0.4.0
[0.3.1]: https://github.com/Blockstream/greenlight/releases/tag/gl-plugin-v0.3.1

