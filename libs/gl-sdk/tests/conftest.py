import glsdk
import pytest


@pytest.fixture
def sdk_config(nobody_id, tmp_path):
    cert = glsdk.DeveloperCert(nobody_id.cert_chain, nobody_id.private_key)
    signer_config = glsdk.SignerConfig(
        glsdk.StateSignatureMode.HARD, str(tmp_path / "device-state")
    )
    return glsdk.Config().with_developer_cert(cert).with_signer_config(signer_config)
