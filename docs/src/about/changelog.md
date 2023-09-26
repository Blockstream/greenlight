# Greenlight Service Changelog

## 2023
### May

 - Calls to `cln.Node/Invoice` now always include all possible
   `routehints`. Possible in this case refers to channels with peers
   that are currently in state `CHANNELD_NORMAL`, both disconnected
   and connected.
   
### June 
 - The JS bindings where clobbering the error messages due to
   incorrect context use. Now we return errors as they are emitted.
 - The scheduler no longer allows creating `regtest` nodes, since they
   are unusable without a faucet to get coins for it.

### July
 - The `gl-plugin` will now wait for both the initial gossip sync and
   the reconnection to the peers to complete, before allowing `pay`
   through. This cuts down on spurious payment failures due to missing
   peers or incomplete network view for routing.
 - An issue concerning reconnecting to peers, if the signer attaches
   before the underlying JSON-RPC has become available has been
   fixed. This issue would cause peers to remain disconnected despite
   a signer being attached. [#210][pr210] & [#204][pr204]
 - The `gl-client` library and the language bindings have keepalive
   messages enabled, with a timeout of 90 seconds. This ensures that
   clients and signers that have been silently disconnected, e.g., by
   suspending the device or losing network connectivity, will notice
   and reconnect. [#220][pr220]
 - The Node Domain has been enabled. This means that ever node now has
   a unique URL at which the node can always be reached, without
   having to explicitly schedule it first. This allows bypassing of
   the scheduler, reducing the time required to start and connect to a
   node.

### September
 - Temporarily removed the JS bindings. We will the bindings over to
   uniffi, and the JS bindings were outdated and unused. But they'll
   be back.

[pr204]: https://github.com/Blockstream/greenlight/pull/204
[pr210]: https://github.com/Blockstream/greenlight/pull/210
[pr220]: https://github.com/Blockstream/greenlight/pull/220
