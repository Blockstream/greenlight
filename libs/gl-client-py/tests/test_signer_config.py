import pytest

import glclient
from glclient import native


@pytest.fixture(autouse=True)
def use_embedded_credentials(monkeypatch):
    for name in ("GL_CA_CRT", "GL_NOBODY_CRT", "GL_NOBODY_KEY"):
        monkeypatch.delenv(name, raising=False)


@pytest.mark.parametrize("constructor", [glclient.Signer, native.Signer])
def test_hard_default_opens_device_storage(constructor, tmp_path):
    path = tmp_path / "device-state"
    signer = constructor(
        b"\x00" * 32,
        "regtest",
        glclient.Credentials(),
        device_storage_path=str(path),
    )
    assert len(bytes(signer.node_id())) == 33
    assert path.is_dir()


@pytest.mark.parametrize("constructor", [glclient.Signer, native.Signer])
@pytest.mark.parametrize("path", [None, ""])
@pytest.mark.parametrize("mode", ["hard", "soft"])
def test_production_modes_require_device_storage(constructor, path, mode):
    with pytest.raises(ValueError, match="device_storage_path"):
        constructor(
            b"\x00" * 32,
            "regtest",
            glclient.Credentials(),
            state_signature_mode=mode,
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
