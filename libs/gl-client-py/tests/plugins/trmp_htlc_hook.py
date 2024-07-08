#!/usr/bin/env python3

from pyln.client import Plugin

plugin = Plugin(
    dynamic=False,
    init_features=1 << 427,
)

@plugin.hook("htlc_accepted")
def on_htlc_accepted(htlc, onion, plugin, **kwargs):
    plugin.log(f"Got onion {onion}")

    # Stip off custom payload as we are the last hop.
    new_payload = onion["payload"][6:102]
    plugin.log(f"Replace onion payload with {new_payload}")
    return {"result": "continue", "payload": new_payload}


plugin.run()
