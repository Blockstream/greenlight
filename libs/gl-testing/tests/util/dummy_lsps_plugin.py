#! /usr/bin/python
from pyln.client import Plugin

# This is a dummy plugin
# A node running this plugin (falsely) advertises itself to be an LSP
# It does not implement any LSP-functionality
plugin = Plugin(
    node_features="0200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    dynamic=False,
)

if __name__ == "__main__":
    plugin.run()
