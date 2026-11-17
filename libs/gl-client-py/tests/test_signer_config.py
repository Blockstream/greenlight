import pytest

import glclient
from glclient import native


@pytest.mark.parametrize("constructor", [glclient.Signer, native.Signer])
def test_hard_default_does_not_open_device_storage(constructor, tmp_path):
    path = tmp_path / "not-a-directory"
    path.write_text("unused")
    signer = constructor(
        b"\x00" * 32,
        "regtest",
        glclient.Credentials(),
        device_storage_path=str(path),
    )
    assert len(bytes(signer.node_id())) == 33


@pytest.mark.parametrize("constructor", [glclient.Signer, native.Signer])
@pytest.mark.parametrize("path", [None, ""])
def test_soft_requires_device_storage(constructor, path):
    with pytest.raises(ValueError, match="requires device_storage_path"):
        constructor(
            b"\x00" * 32,
            "regtest",
            glclient.Credentials(),
            state_signature_mode="soft",
            device_storage_path=path,
        )


@pytest.mark.parametrize("constructor", [glclient.Signer, native.Signer])
def test_soft_opens_persistent_storage(constructor, tmp_path):
    path = tmp_path / "device-state"
    for _ in range(2):
        signer = constructor(
            b"\x00" * 32,
            "regtest",
            glclient.Credentials(),
            state_signature_mode="soft",
            device_storage_path=str(path),
        )
        assert len(bytes(signer.node_id())) == 33
        assert path.is_dir()


def test_unknown_state_signature_mode_is_rejected():
    with pytest.raises(ValueError, match="Unknown state signature mode"):
        glclient.Signer(
            b"\x00" * 32,
            "regtest",
            glclient.Credentials(),
            state_signature_mode="typo",
        )


def test_storage_errors_reach_python(tmp_path):
    path = tmp_path / "not-a-directory"
    path.write_text("unavailable")
    with pytest.raises(ValueError, match="opening device storage"):
        glclient.Signer(
            b"\x00" * 32,
            "regtest",
            glclient.Credentials(),
            state_signature_mode="soft",
            device_storage_path=str(path),
        )
