import importlib.metadata
from clnvm.cln_version_manager import ClnVersionManager

__version__ = importlib.metadata.version("cln-version-manager")

__all__ = ["ClnVersionManager"]

