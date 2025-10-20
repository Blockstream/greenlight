# Changelog

All notable changes to the subprojects will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.3.1] - 2025-07-14

### Added

 - `gl-client` 0.3.1 released, including the signer for CLN
   v.25.05. Upgrading the client and signer will trigger an upgrade on
   the server, and you will be using the latest and greatest CLN
   version going forward. 
 - Added support to `trampoline_pay`, a payment exection engine that
   delegates route selection and payment execution to the LSP node.
 - Several signer policies have been relaxed in order to prevent
   signer rejections. Some rejections are still happening due to a
   mismatch in CLN and VLS data models, but most should be fixed.
 - A standalone `gl-testserver` subproject allows testing non-python clients by spinning up a mock Greenlight system in a box, and then run tests against it. ([#539][PR539])
 - A python-specific subprohect `gl-testing` allows writing extensive `pytest` tests against Greenlight. If you are using Python, this is a great way to test your project
 - Added CLN v24.11 as supported version on the server.
 - Added a rust-based `glcli` command line client replacing the python `glcli`

### Changed

 - `trampoline_pay` performs a preflight check to see if we have enough `spendable_msat` before attempting the trampoline payment. ([#585][PR585])

### Fixed

 - Fixed a logic error relating to zeroconf channels, and forgetting channels that did not confirm for prolonged periods.


 
## [0.3.0] - 2024-10-07

### Added

- Capture and report signer rejections back to the node ([#483](https://github.com/Blockstream/greenlight/pull/483), #[484](https://github.com/Blockstream/greenlight/pull/484)).
- Add trampoline client to delegate route finding to next node ([#475](https://github.com/Blockstream/greenlight/pull/475), #[489](https://github.com/Blockstream/greenlight/pull/489), #[498](https://github.com/Blockstream/greenlight/pull/498), #[505](https://github.com/Blockstream/greenlight/pull/505), #[511](https://github.com/Blockstream/greenlight/pull/511)).
- Add basic support for signer-less devices ([#281](https://github.com/Blockstream/greenlight/pull/281)).

[PR539]: https://github.com/Blockstream/greenlight/pull/539

### Changed

- Ensure that signer doesn't exit on network change ([#524](https://github.com/Blockstream/greenlight/pull/524))
- Signer reports the node id by itself ([#520](https://github.com/Blockstream/greenlight/pull/520))
- Upgrade to VLS 0.12 ([#504](https://github.com/Blockstream/greenlight/pull/504)).
- Add .resources dir to all crates in repo. This is soley to make it possible to publish artefacts ([#501](https://github.com/Blockstream/greenlight/pull/501))

[PR585]: https://github.com/Blockstream/greenlight/pull/585

### Fixed

- Several small drive-by fixes ([#468](https://github.com/Blockstream/greenlight/pull/468), [#471](https://github.com/Blockstream/greenlight/pull/471), [#508](https://github.com/Blockstream/greenlight/pull/508))

### Removed

- The methods that used to be in `greenlight.proto` that have been
  superseded with the `node.proto` version have been removed on the
  server side. The proto file now contains only Greenlight-specific
  functionality ([#317][pr317].

## [0.2.0]

### Added

 - The Node Domain has been enabled. This means that ever node now has
   a unique URL at which the node can always be reached, without
   having to explicitly schedule it first. This allows bypassing of
   the scheduler, reducing the time required to start and connect to a
   node.

 - The plugin now wait for the node to complete its startup before
   forwarding RPC commands.
 - Calls to `cln.Node/Invoice` now always include all possible
   `routehints`. Possible in this case refers to channels with peers
   that are currently in state `CHANNELD_NORMAL`, both disconnected
   and connected.
 - The `gl-plugin` will now wait for both the initial gossip sync and
   the reconnection to the peers to complete, before allowing `pay`
   through. This cuts down on spurious payment failures due to missing
   peers or incomplete network view for routing.
 - The `gl-client` library and the language bindings have keepalive
   messages enabled, with a timeout of 90 seconds. This ensures that
   clients and signers that have been silently disconnected, e.g., by
   suspending the device or losing network connectivity, will notice
   and reconnect. [#220][pr220]

### Fixed

 - An issue concerning reconnecting to peers, if the signer attaches
   before the underlying JSON-RPC has become available has been
   fixed. This issue would cause peers to remain disconnected despite
   a signer being attached. [#210][pr210] & [#204][pr204]

### Removed

 - The JS bindings where clobbering the error messages due to
   incorrect context use. Now we return errors as they are emitted.
 - The scheduler no longer allows creating `regtest` nodes, since they
   are unusable without a faucet to get coins for it.

 - Temporarily removed the JS bindings. We will the bindings over to
   uniffi, and the JS bindings were outdated and unused. But they'll
   be back.
 - The API has been simplified by removing methods that were both in
   `greenlight.proto` as well as `node.proto`. The latter is from the
   autogenerated `cln-grpc` which supercedes the `greenlight.proto`
   methods.

 - Payment optimizations: we are working on getting the success rate
   for payments up, and the time to completion down, focusing on
   success rate first.

[pr204]: https://github.com/Blockstream/greenlight/pull/204
[pr210]: https://github.com/Blockstream/greenlight/pull/210
[pr220]: https://github.com/Blockstream/greenlight/pull/220
[pr317]:  https://github.com/Blockstream/greenlight/pull/317
