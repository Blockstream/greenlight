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


def test_connect_fails_with_empty_creds():
    """Connecting with empty credentials must error."""
    config = glsdk.Config()
    mnemonic = (
        "abandon abandon abandon abandon abandon abandon "
        "abandon abandon abandon abandon abandon about"
    )
    with pytest.raises(glsdk.Error):
        glsdk.connect(mnemonic, b"", config)


def test_developer_cert_construction():
    """Test that DeveloperCert can be constructed with cert and key bytes."""
    cert = glsdk.DeveloperCert(b"fake-cert-pem", b"fake-key-pem")
    assert cert is not None
    assert isinstance(cert, glsdk.DeveloperCert)


def test_developer_cert_type_error():
    """Test that passing wrong types to DeveloperCert raises TypeError."""
    with pytest.raises(TypeError):
        glsdk.DeveloperCert("not bytes", b"key")
    with pytest.raises(TypeError):
        glsdk.DeveloperCert(b"cert", "not bytes")


def test_scheduler_with_developer_cert():
    """Test that with_developer_cert returns a new Scheduler instance."""
    cert = glsdk.DeveloperCert(b"fake-cert-pem", b"fake-key-pem")
    scheduler = glsdk.Scheduler(glsdk.Network.BITCOIN)
    scheduler_with_cert = scheduler.with_developer_cert(cert)

    # Should return a new Scheduler instance, not modify the original
    assert scheduler_with_cert is not None
    assert isinstance(scheduler_with_cert, glsdk.Scheduler)


def test_register_with_developer_cert(scheduler, nobody_id):
    """Test that register works when using an explicit DeveloperCert."""
    # Load the test nobody cert/key from the fixture's byte attributes
    dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)

    signer = glsdk.Signer(
        "abandon abandon abandon abandon abandon abandon "
        "abandon abandon abandon abandon abandon about"
    )
    s = glsdk.Scheduler(glsdk.Network.BITCOIN).with_developer_cert(dev_cert)
    creds = s.register(signer, code=None)
    assert creds is not None
    assert isinstance(creds, glsdk.Credentials)


def test_register_and_auth(scheduler, clients):
    signer = glsdk.Signer(
        "abandon abandon abandon abandon abandon abandon "
        "abandon abandon abandon abandon abandon about"
    )

    # Now use the signer to sign up a new node:
    scheduler = glsdk.Scheduler(glsdk.Network.BITCOIN)
    creds = scheduler.register(signer, code=None)
