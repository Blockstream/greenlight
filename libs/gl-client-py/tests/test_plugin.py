from gltesting.fixtures import Client
from gltesting.fixtures import *
import secrets

def test_big_size_requests(clients):
    """We want to test if we can pass through big size (up to ~4MB)
    requests. These requests are handled by the gl-plugin to extract
    the request context that is passed to the signer.
    We need to test if the request is fully captured and passed to 
    cln-grpc.
    """
    c1: Client = clients.new()
    c1.register()
    n1 = c1.node()
    # Size is roughly 4MB with some room for grpc overhead.
    size = 3990000
    
    # Write large data to the datastore.
    n1.datastore("some-key", hex=bytes.fromhex(secrets.token_hex(size)))

def test_max_message_size(clients):
    """Tests that the maximum message size is ensured by the plugin.
    This is currently hard-coded to 4194304bytes. The plugin should
    return with an grpc error.
    """
    c1: Client = clients.new()
    c1.register()
    n1 = c1.node()
    size = 4194304 + 1
    
    # Send message too large.
    with pytest.raises(ValueError):
        n1.datastore("some-key", hex=bytes.fromhex(secrets.token_hex(size)))