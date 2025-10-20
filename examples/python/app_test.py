import os
import pytest
from glclient import Scheduler, Signer, TlsConfig,Node, nodepb

class GetInfoApp:
    """An example application for gltesting.
    
    This example application shows the process on how to register,
    scheduler and call against a gltesting environment greenlight 
    node.

    To execute this example set up the docker gltesting environment,
    drop into a repl as explained in the gltesting tutorial.

    Then run the test below outside the gltesting docker container
    (run it from the host).
    `pytest -s -v app_test.py::test_getinfoapp`.
    """
    def __init__(self, secret: bytes, network: str, tls: TlsConfig):
        self.secret: bytes = secret
        self.network = network
        self.tls: TlsConfig = tls
        self.signer: Signer = Signer(secret, network, tls) # signer needs to keep running
        self.node_id: bytes = self.signer.node_id()

    def scheduler(self) -> Scheduler:
        """Returns a glclient Scheduler

        The scheduler is created from the attributes stored in this
        class.
        """
        return Scheduler(self.node_id, self.network, self.tls)

    def register_or_recover(self):
        """Registers or recovers a node on gltesting
        
        Also sets the new identity after register/recover.
        """
        res = None
        try:
            res = self.scheduler().register(self.signer)
        except:
            res = self.scheduler().recover(self.signer)
        
        self.tls = self.tls.identity(res.device_cert, res.device_key)

    def get_info(self) -> nodepb.GetInfoResponse:
        """Requests getinfo on the gltesting greenlight node"""
        res = self.scheduler().schedule()
        node = Node(self.node_id, self.tls, res.grpc_uri)
        return node.get_info()


def test_getinfoapp():
    # These are normally persisted on disk and need to be loaded and
    # passed to the glclient library by the application. In this 
    # example we store them directly in the "app".
    secret = b'\x00'*32
    network='regtest'
    tls = TlsConfig()

    # Register a node
    giap = GetInfoApp(secret, network, tls)
    giap.register_or_recover()

    # GetInfo
    res = giap.get_info()
    print(f"res={res}")