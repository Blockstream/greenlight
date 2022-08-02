from gltesting.identity import Identity
from gltesting.utils import NodeVersion
from binascii import hexlify
from glclient import greenlight_pb2_grpc as nodegrpc
from glclient import greenlight_pb2 as nodepb
import logging
import grpc
from concurrent.futures import ThreadPoolExecutor


class Node:
    """A node running under the control of a scheduler.

    Clients can control it over the grpc plugin, and signers can
    attach to provide signatures when required.
    """

    def __init__(self, identity: Identity, version: NodeVersion):
        self.identity = identity
        self.version  = version

    def start(self):
        pass

    def stop(self):
        pass


class MockNode:
    """Just a mock node replying to queries with the default response.
    """

    def __init__(self, node_id, identity, grpc_port):
        self.identity = identity
        self.node_id = node_id
        self.hex_node_id = hexlify(self.node_id)
        self.grpc_port = grpc_port
        self.server = None

    def start(self):
        logging.info(f"Starting node={self.hex_node_id} on port {self.grpc_port}")
        if self.server is not None:
            raise ValueError("Server already running")

        cred = self.identity.to_server_credentials()
        self.server = grpc.server(ThreadPoolExecutor(max_workers=1))
        nodegrpc.add_NodeServicer_to_server(self, self.server)
        self.server.add_secure_port(f'[::]:{self.grpc_port}', cred)
        self.server.start()
        logging.info(f"Scheduler started on port {self.grpc_port}")

    def stop(self):
        pass

    @property
    def grpc_addr(self):
        return f"https://localhost:{self.grpc_port}"

    def GetInfo(self, req, ctx):
        return nodepb.GetInfoResponse(
            node_id=self.node_id,
        )

    def Stop(self, req, ctx):
        raise ValueError() # returns (StopResponse) {}
    def ConnectPeer(self, req, ctx):
        raise ValueError() # returns (ConnectResponse) {}
    def ListPeers(self, req, ctx):
        raise ValueError() # returns (ListPeersResponse) {}
    def Disconnect(self, req, ctx):
        raise ValueError() # returns (DisconnectResponse) {}
    def NewAddr(self, req, ctx):
        raise ValueError() # returns (NewAddrResponse) {}
    def ListFunds(self, req, ctx):
        raise ValueError() # returns (ListFundsResponse) {}
    def Withdraw(self, req, ctx):
        raise ValueError() # returns (WithdrawResponse) {}
    def FundChannel(self, req, ctx):
        raise ValueError() # returns (FundChannelResponse) {}
    def CloseChannel(self, req, ctx):
        raise ValueError() # returns (CloseChannelResponse) {}
    def CreateInvoice(self, req, ctx):
        raise ValueError() # returns (Invoice) {}
    def Pay(self, req, ctx):
        raise ValueError() # returns (Payment) {}
    def Keysend(self, req, ctx):
        raise ValueError() # returns (Payment) {}
    def ListPayments(self, req, ctx):
        raise ValueError() # returns (ListPaymentsResponse) {}
    def ListInvoices(self, req, ctx):
        raise ValueError() # returns (ListInvoicesResponse) {}
    def StreamIncoming(self, req, ctx):
        raise ValueError() # returns (stream IncomingPayment) {}
    def StreamLog(self, req, ctx):
        raise ValueError() # returns (stream LogEntry) {}
    def StreamHsmRequests(self, req, ctx):
        raise ValueError() # returns (stream HsmRequest) {}
    def RespondHsmRequest(self, req, ctx):
        raise ValueError() # returns (Empty) {}
