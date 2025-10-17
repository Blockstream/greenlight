"""Tests for gl-sdk Credentials wrapper around gl-client credentials.

This test suite validates that the UniFFI-exposed Credentials struct
correctly wraps the underlying gl-client Device credentials and can
load credentials data from disk.
"""
import pytest
import glsdk
from gltesting.fixtures import *


def test_credentials_load_from_registered_client(scheduler, clients):
    """Load credentials from a registered client's greenlight.auth file.

    This test verifies that after a client registers with the scheduler,
    the credentials file can be successfully loaded via the gl-sdk
    Credentials.load() method exposed through UniFFI.
    """
    # Create a new client and register it
    c = clients.new()
    c.register(configure=True)

    # Verify the credentials file was created
    creds_path = c.directory / "greenlight.auth"
    assert creds_path.exists(), "greenlight.auth file should exist after registration"

    # Read the credentials file
    creds_bytes = creds_path.read_bytes()
    assert len(creds_bytes) > 0, "Credentials file should not be empty"

    # Load credentials using the gl-sdk UniFFI wrapper
    credentials = glsdk.Credentials.load(creds_bytes)

    # Verify we got a valid Credentials object
    assert credentials is not None
    assert isinstance(credentials, glsdk.Credentials)


def test_credentials_load_empty_data(scheduler, clients):
    """Verify that loading credentials with empty data doesn't crash.

    The underlying gl-client Device::from_bytes implementation defaults
    to nobody credentials when given invalid/empty data, so this should
    succeed but return a default credentials object.
    """
    # Create empty credentials data
    empty_data = b""

    # This should not crash - Device::from_bytes defaults to nobody creds
    credentials = glsdk.Credentials.load(empty_data)
    assert credentials is not None
    assert isinstance(credentials, glsdk.Credentials)


def test_credentials_load_invalid_data(scheduler, clients):
    """Test that loading completely invalid data doesn't crash.

    The underlying implementation should handle invalid data gracefully
    by falling back to default nobody credentials.
    """
    # Create some random invalid data
    invalid_data = b"not valid credentials data at all!"

    # This should not crash
    credentials = glsdk.Credentials.load(invalid_data)
    assert credentials is not None


def test_credentials_multiple_loads(scheduler, clients):
    """Verify that the same credentials data can be loaded multiple times.

    This ensures there are no issues with resource ownership or
    cleanup when creating multiple Credentials instances from the same data.
    """
    c = clients.new()
    c.register(configure=True)

    creds_bytes = (c.directory / "greenlight.auth").read_bytes()

    # Load the same credentials multiple times
    creds1 = glsdk.Credentials.load(creds_bytes)
    creds2 = glsdk.Credentials.load(creds_bytes)
    creds3 = glsdk.Credentials.load(creds_bytes)

    # All should be valid Credentials objects
    assert isinstance(creds1, glsdk.Credentials)
    assert isinstance(creds2, glsdk.Credentials)
    assert isinstance(creds3, glsdk.Credentials)


def test_credentials_from_recovered_client(scheduler, clients):
    """Test loading credentials from a recovered client.

    This verifies that credentials work correctly through the
    recovery flow, not just registration.
    """
    # Create a client and register
    secret = bytes([42] * 32)
    c1 = clients.new(secret=secret)
    c1.register(configure=False)

    # Create a new client with the same secret and recover
    c2 = clients.new(secret=secret)
    c2.recover(configure=True)

    # Load credentials from the recovered client
    creds_path = c2.directory / "greenlight.auth"
    assert creds_path.exists()

    creds_bytes = creds_path.read_bytes()
    credentials = glsdk.Credentials.load(creds_bytes)

    assert credentials is not None
    assert isinstance(credentials, glsdk.Credentials)


# Intentionally failing test to demonstrate error handling
def test_credentials_type_error_fails():
    """This test should fail: passing wrong type to Credentials.load().

    UniFFI should raise a TypeError when we pass a string instead of bytes.
    This test demonstrates proper error handling and type checking.
    """
    with pytest.raises(TypeError) as exc_info:
        # This should fail - we're passing a string, not bytes
        glsdk.Credentials.load("this is a string, not bytes")

    # Verify we get a helpful error message about type mismatch
    assert "bytes" in str(exc_info.value).lower() or "type" in str(exc_info.value).lower()
