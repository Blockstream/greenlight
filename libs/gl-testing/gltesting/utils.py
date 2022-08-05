from dataclasses import dataclass
from enum import Enum

@dataclass
class NodeVersion:
    # The path that we should use when calling `lightningd`, it is
    # version specific
    path: str
    # The stringified version number
    name: str


@dataclass
class SignerVersion:
    name: str

    def is_compat(self, nv: NodeVersion) -> bool:
        """Whether or not a signer is compatible with a given NodeVersion."""
        compat = {
            "v0.10.1": ["v0.10.1"],
            "v0.10.2": ["v0.10.2"],
            "v0.11.0.1": ["v0.11.0.1", "v0.11.2gl2"],
            "v0.11.2": ["v0.11.0.1", "v0.11.2gl2"],
        }

        return self.name in compat[nv.name]

    def get_node_version(self):
        """Return the node version we should start for this signer version.
        """
        m = {
            "v0.10.1": "v0.10.1",
            "v0.10.2": "v0.10.2",
            "v0.11.0.1": "v0.11.2gl2",
            "v0.11.2": "v0.11.2gl2",
        }
        return m[self.name]


class Network(Enum):
    """Supported networks."""

    BITCOIN = 0
    TESTNET = 1
    REGTEST = 2
