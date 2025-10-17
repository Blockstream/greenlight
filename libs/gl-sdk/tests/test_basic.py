"""Basic tests for gl-sdk without requiring full scheduler setup.

These tests verify the UniFFI bindings work correctly without
needing the full gl-testing infrastructure.
"""

from gltesting.fixtures import *
import pytest
import glsdk


def test_import_glsdk():
    """Test that glsdk can be imported."""
    assert glsdk is not None
    assert hasattr(glsdk, "Credentials")
    assert hasattr(glsdk, "Node")
    assert hasattr(glsdk, "Signer")
    assert hasattr(glsdk, "Error")


def test_credentials_load_empty():
    """Test loading credentials with empty data doesn't crash."""
    # This should not crash - Device::from_bytes defaults to nobody creds
    credentials = glsdk.Credentials.load(b"")
    assert credentials is not None
    assert isinstance(credentials, glsdk.Credentials)


def test_credentials_load_invalid():
    """Test loading credentials with invalid data doesn't crash."""
    # This should not crash either
    credentials = glsdk.Credentials.load(b"invalid data")
    assert credentials is not None


def test_credentials_type_error():
    """Test that passing wrong type raises TypeError."""
    with pytest.raises(TypeError):
        glsdk.Credentials.load("not bytes")


def test_credentials_multiple_loads():
    """Test that we can create multiple credentials objects."""
    creds1 = glsdk.Credentials.load(b"test1")
    creds2 = glsdk.Credentials.load(b"test2")
    creds3 = glsdk.Credentials.load(b"test3")

    assert all(isinstance(c, glsdk.Credentials) for c in [creds1, creds2, creds3])


def test_node_creation_fails_with_empty_creds():
    """Test that creating a Node with empty credentials fails as expected."""
    creds = glsdk.Credentials.load(b"")

    # Node creation should fail with these invalid credentials
    with pytest.raises(glsdk.Error):
        node = glsdk.Node(creds)


def test_register_and_auth(scheduler, clients):
    signer = glsdk.Signer(
        "abandon abandon abandon abandon abandon abandon "
        "abandon abandon abandon abandon abandon about"
    )

    # Now use the signer to sign up a new node:
    scheduler = glsdk.Scheduler(glsdk.Network.BITCOIN)
    creds = scheduler.register(signer, code=None)
