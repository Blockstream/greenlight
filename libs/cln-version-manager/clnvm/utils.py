from dataclasses import dataclass
from pathlib import Path


@dataclass
class NodeVersion:
    """Path to the lightningd executable"""

    path: Path
    """Name of the version"""
    name: str
