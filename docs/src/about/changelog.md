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
