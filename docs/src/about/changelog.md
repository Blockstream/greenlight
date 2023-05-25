# Greenlight Service Changelog

## 2023
### May

 - Calls to `cln.Node/Invoice` now always include all possible
   `routehints`. Possible in this case refers to channels with peers
   that are currently in state `CHANNELD_NORMAL`, both disconnected
   and connected.
