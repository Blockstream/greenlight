import importlib.metadata
from clnvm.cln_version_manager import ClnVersionManager

version = importlib.metadata.version("cln-version-manager")

__all__ = ["ClnVersionManager"]
