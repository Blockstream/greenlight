from dataclasses import dataclass
from pathlib import Path


@dataclass
class NodeVersion:
    """Contains version information and executable to a node.

    It also includes the path used to find the executable
    """

    """Path to the lightningd executable"""
    lightningd: Path
    """Path to the bin-folder of the release"""
    bin_path: Path
    """Path to the root folder of the release.
    Typically, this contains the `usr` directory"""
    root_path: Path
    """Name of the version"""
    name: str
