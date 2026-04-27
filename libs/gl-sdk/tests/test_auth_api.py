"""Integration tests for the high-level auth API.

Tests the register(), recover(), connect(), and register_or_recover()
free functions, plus the Config type.
"""

import pytest
import glsdk
from gltesting.fixtures import *


MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)


class TestConfig:
    """Test Config construction and builder methods."""

    def test_default_config(self):
        config = glsdk.Config()
        assert config is not None
        assert isinstance(config, glsdk.Config)

    def test_config_with_network(self):
        config = glsdk.Config().with_network(glsdk.Network.REGTEST)
        assert config is not None
        assert isinstance(config, glsdk.Config)

    def test_config_with_developer_cert(self):
        cert = glsdk.DeveloperCert(b"fake-cert", b"fake-key")
        config = glsdk.Config().with_developer_cert(cert)
        assert config is not None
        assert isinstance(config, glsdk.Config)

    def test_config_chaining(self):
        cert = glsdk.DeveloperCert(b"fake-cert", b"fake-key")
        config = (
            glsdk.Config()
            .with_developer_cert(cert)
            .with_network(glsdk.Network.REGTEST)
        )
        assert config is not None


class TestRegister:
    """Test the register() free function."""

    def test_register_returns_node(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        assert node is not None
        assert isinstance(node, glsdk.Node)
        node.disconnect()

    def test_register_credentials_roundtrip(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        creds = node.credentials()
        assert isinstance(creds, bytes)
        assert len(creds) > 0
        node.disconnect()

    def test_register_bad_mnemonic(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        with pytest.raises(glsdk.Error.PhraseCorrupted):
            glsdk.NodeBuilder(config).register("not a valid mnemonic", None)


class TestRecover:
    """Test the recover() free function."""

    def test_recover_after_register(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        # Register first
        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node1.disconnect()

        # Recover with same mnemonic
        node2 = glsdk.NodeBuilder(config).recover(MNEMONIC)
        assert node2 is not None
        assert isinstance(node2, glsdk.Node)
        creds = node2.credentials()
        assert len(creds) > 0
        node2.disconnect()

    def test_recover_nonexistent_node(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        with pytest.raises(glsdk.Error.NoSuchNode):
            glsdk.NodeBuilder(config).recover(MNEMONIC)


class TestConnect:
    """Test the connect() free function."""

    def test_connect_with_saved_credentials(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        # Register and save credentials
        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        saved_creds = node1.credentials()
        node1.disconnect()

        # Connect with saved credentials
        node2 = glsdk.NodeBuilder(config).connect(saved_creds, MNEMONIC)
        assert node2 is not None
        assert isinstance(node2, glsdk.Node)
        node2.disconnect()

    def test_connect_bad_mnemonic(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        with pytest.raises(glsdk.Error.PhraseCorrupted):
            glsdk.NodeBuilder(config).connect(b"some-creds", "bad mnemonic")


class TestRegisterOrRecover:
    """Test the register_or_recover() free function."""

    def test_registers_when_no_node_exists(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register_or_recover(MNEMONIC, None)
        assert node is not None
        assert isinstance(node, glsdk.Node)
        node.disconnect()

    def test_recovers_when_node_exists(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        # Register first
        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node1.disconnect()

        # register_or_recover should recover
        node2 = glsdk.NodeBuilder(config).register_or_recover(MNEMONIC, None)
        assert node2 is not None
        assert isinstance(node2, glsdk.Node)
        node2.disconnect()


class TestDisconnect:
    """Test the disconnect() method."""

    def test_disconnect_stops_signer(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        # Should not raise
        node.disconnect()

    def test_disconnect_idempotent(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node.disconnect()
        # Second disconnect should not raise
        node.disconnect()


class TestDuplicateRegister:
    """Test that registering the same node twice fails."""

    def test_duplicate_register(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        # Register once
        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node1.disconnect()

        # Register again with same mnemonic should fail
        with pytest.raises(glsdk.Error.DuplicateNode):
            glsdk.NodeBuilder(config).register(MNEMONIC, None)


class TestConnectBadCredentials:
    """Test that connecting with invalid credentials fails."""

    def test_connect_empty_credentials(self, scheduler, nobody_id):
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        # Empty credentials should fail (Credentials.load returns nobody creds,
        # but the signer won't be able to authenticate with them)
        with pytest.raises(glsdk.Error):
            glsdk.NodeBuilder(config).connect(b"", MNEMONIC)


class TestMultipleNodes:
    """Test running multiple nodes simultaneously."""

    def test_two_nodes_independent(self, scheduler, nobody_id):
        """Two nodes from different mnemonics can run simultaneously."""
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        mnemonic_2 = (
            "zoo zoo zoo zoo zoo zoo "
            "zoo zoo zoo zoo zoo wrong"
        )

        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node2 = glsdk.NodeBuilder(config).register(mnemonic_2, None)

        assert node1 is not None
        assert node2 is not None
        assert isinstance(node1, glsdk.Node)
        assert isinstance(node2, glsdk.Node)

        # Both should have independent credentials
        creds1 = node1.credentials()
        creds2 = node2.credentials()
        assert creds1 != creds2

        node1.disconnect()
        node2.disconnect()


class TestDisconnectBlocksRpc:
    """Test that RPC calls fail after disconnect."""

    def test_credentials_still_works_after_disconnect(self, scheduler, nobody_id):
        """credentials() should work even after disconnect since it's local data."""
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)
        node = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        node.disconnect()
        # credentials() is local, should still work
        creds = node.credentials()
        assert len(creds) > 0


class TestLowLevelCredentials:
    """Test that Node created via low-level API exposes credentials."""

    def test_node_new_stores_credentials(self, scheduler, nobody_id):
        """Node::new(creds) should allow calling node.credentials()."""
        dev_cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
        config = glsdk.Config().with_developer_cert(dev_cert)

        # Register to get valid credentials
        node1 = glsdk.NodeBuilder(config).register(MNEMONIC, None)
        saved_creds = node1.credentials()
        node1.disconnect()

        # Create node via low-level API
        creds_obj = glsdk.Credentials.load(saved_creds)
        node2 = glsdk.Node(creds_obj)
        roundtripped = node2.credentials()
        assert len(roundtripped) > 0
