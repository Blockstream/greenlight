from pathlib import Path
from .scheduler import Scheduler


def get_plugins_dir():
    """Get the path to the plugins directory.

    This works both when running from source (where plugins is a sibling
    of gltesting) and when installed via pip (where both gltesting and
    plugins are installed into site-packages).

    Returns:
        Path: The absolute path to the plugins directory

    Raises:
        FileNotFoundError: If the plugins directory cannot be found
    """
    # Get the directory containing this __init__.py file (gltesting/)
    package_dir = Path(__file__).parent

    # Try to find plugins as a sibling directory
    # This handles both source and installed cases since both directories
    # are installed at the same level
    plugins_dir = package_dir.parent / "plugins"

    if plugins_dir.exists() and plugins_dir.is_dir():
        return plugins_dir

    raise FileNotFoundError(
        f"Could not find plugins directory. Expected at: {plugins_dir}"
    )


__all__ = [
    "Scheduler",
    "get_plugins_dir",
]
