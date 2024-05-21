from dataclasses import dataclass
from enum import Enum
from pathlib import Path

from clnvm.utils import NodeVersion

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
            "v22.11": ["v22.11gl1"],
            "v23.05": ["v23.05gl1"],
            "v23.08": ["v23.08gl1"],
            "v24.02": ["v24.02gl1"],
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
            "v22.11": "v22.11gl1",
            "v23.05": "v23.05gl1",
            "v23.08": "v23.08gl1",
            "v24.02": "v24.02gl1",
        }
        return m[self.name]


class Network(Enum):
    """Supported networks.

    Matches the enum in the `bitcoin::Network` rust code.
    """

    BITCOIN = 0
    TESTNET = 1
    SIGNET = 2
    REGTEST = 3
