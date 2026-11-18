import pytest

import glsdk


MNEMONIC = (
    "abandon abandon abandon abandon abandon abandon "
    "abandon abandon abandon abandon abandon about"
)


def migration_config(path=None):
    return glsdk.SignerConfig(
        state_signature_mode=glsdk.StateSignatureMode.SOFT,
        device_storage_path=str(path) if path is not None else None,
    )


def test_default_signer_constructors_open_storage(tmp_path):
    path = tmp_path / "device-state"
    assert len(glsdk.Signer(MNEMONIC, str(path)).node_id()) == 33
    assert len(glsdk.Signer.new_from_seed(b"\x00" * 32, str(path)).node_id()) == 33
    assert path.is_dir()


@pytest.mark.parametrize("from_seed", [False, True])
def test_configured_signer_constructors(from_seed, tmp_path):
    path = tmp_path / "device-state"
    config = migration_config(path)
    if from_seed:
        signer = glsdk.Signer.new_from_seed_with_config(b"\x00" * 32, config)
    else:
        signer = glsdk.Signer.new_with_config(MNEMONIC, config)
    assert len(signer.node_id()) == 33
    assert path.is_dir()


def test_authentication_keeps_the_signer_identity(tmp_path):
    path = tmp_path / "device-state"
    signer = glsdk.Signer.new_with_config(MNEMONIC, migration_config(path))
    creds = glsdk.Credentials.load(b"")
    authenticated = signer.authenticate(creds)
    assert authenticated.node_id() == signer.node_id()


@pytest.mark.parametrize(
    "action", ["register", "recover", "connect", "register_or_recover"]
)
def test_node_builder_forwards_signer_configuration(action, monkeypatch):
    # A regression must never send these tests to a production scheduler.
    monkeypatch.setenv("GL_SCHEDULER_GRPC_URI", "https://localhost:0")
    config = glsdk.Config().with_signer_config(migration_config())
    builder = glsdk.NodeBuilder(config)
    with pytest.raises(glsdk.Error, match="device_storage_path"):
        if action in ("register", "register_or_recover"):
            getattr(builder, action)(MNEMONIC, None)
        elif action == "connect":
            builder.connect(b"", MNEMONIC)
        else:
            builder.recover(MNEMONIC)
