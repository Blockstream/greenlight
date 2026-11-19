"""Greenlight SDK - UniFFI Python bindings for Greenlight.

This package provides Python bindings to the Greenlight Lightning Network
client library, generated using Mozilla's UniFFI framework.
"""

from .glsdk import *  # noqa: F401, F403

from importlib.metadata import PackageNotFoundError, version as _version

try:
    __version__ = _version("gl-sdk")
except PackageNotFoundError:  # imported from a source checkout
    __version__ = "unknown"
