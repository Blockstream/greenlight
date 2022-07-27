from gltesting.fixtures import *
from fixtures import *
from glclient import Node
from binascii import hexlify


def test_sni_extension(capfd, signer, tls):
    """Check that we write the hostname to the host_name field in the 
    client_handshake message.
    """

    hostname = "gl.blckstrm.com"
    grpc_uri = f"https://{hostname}:1111"

    with pytest.raises(ValueError):
        Node(signer.node_id(), 'regtest', tls, grpc_uri)


    captured = capfd.readouterr()
    assert f"Setting tls host_name {hostname}" in captured.err