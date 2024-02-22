import logging
import os
import random
import shutil
import socket
import subprocess
import string
import tempfile
import threading
import time
from dataclasses import dataclass
from pathlib import Path
from threading import Condition
from typing import Dict, List, Optional

import anyio
import purerpc
from glclient import greenlight_pb2 as greenlightpb
from glclient import scheduler_pb2 as schedpb
from pyln.client import LightningRpc
from pyln.testing.utils import BitcoinD

from gltesting import certs
from gltesting import scheduler_grpc as schedgrpc
from gltesting.identity import Identity
from gltesting.node import NodeProcess
from gltesting.utils import Network, NodeVersion, SignerVersion

from clnvm import ClnVersionManager

@dataclass
class Node:
    node_id: bytes
    signer_version: SignerVersion
    directory: Path
    network: Network
    initmsg: bytes
    identity: Identity
    process: Optional[NodeProcess]
    plugin_grpc_uri: Optional[str]
    startupmsgs: List[schedpb.StartupMessage]
    # Condition we wait on in GetNodeInfo for signers
    condition: Condition

    def rpc(self) -> LightningRpc:
        return LightningRpc(self.directory / "regtest" / "lightning-rpc")


@dataclass
class Challenge:
    node_id: bytes
    challenge: bytes
    scope: str
    used: bool


@dataclass
class InviteCode:
    code: str
    is_redeemed: bool


def enumerate_cln_versions() -> Dict[str, NodeVersion]:
    """Search `$PATH` and `$CLN_PATH` for CLN versions."""
    manager = ClnVersionManager()
    return manager.get_all()
    

def generate_secret(len=5):
    return "".join(random.choices(string.ascii_uppercase, k=len))

class AsyncScheduler(schedgrpc.SchedulerServicer):
    """A mock scheduler to test against."""

    def __init__(
        self, bitcoind: BitcoinD, grpc_port=443, identity=None, node_directory=None
    ):
        self.grpc_port = grpc_port
        self.server = None
        print("Starting scheduler with caroot", identity.caroot)
        self.identity = identity
        self.challenges: List[Challenge] = []
        self.next_challenge: int = 1
        self.nodes: List[Node] = []
        self.versions = enumerate_cln_versions()
        self.bitcoind = bitcoind
        self.invite_codes: List[str] = []
        self.next_webhook_id: int = 1
        self.received_invite_code = None
        self.debugger = DebugServicer()
        self.webhooks = []
        self.pairings = PairingServicer()

        if node_directory is not None:
            self.node_directory = node_directory
        else:
            self.node_directory = Path(tempfile.mkdtemp())

    @property
    def grpc_addr(self):
        return f"https://localhost:{self.grpc_port}"

    async def run(self):
        """Entrypoint for the async runtime to take over."""
        await self.server.serve_async()

    def start(self):
        logging.info(f"Starting scheduler on port {self.grpc_port}")
        if self.server is not None:
            raise ValueError("Server already running")
        self.server = purerpc.Server(
            port=self.grpc_port, ssl_context=self.identity.to_ssl_context()
        )
        self.server.add_service(self.service)
        self.server.add_service(self.debugger.service)
        self.server.add_service(self.pairings.service)

        threading.Thread(target=anyio.run, args=(self.run,), daemon=True).start()
        logging.info(f"Scheduler started on port {self.grpc_port}")

    def stop(self):
        # TODO Find a way to stop the server gracefully. Restarting
        # the xdist worker after each test might also address this.
        self.server = None
        for n in self.nodes:
            if n.process:
                n.process.stop()
                n.process = None

    def get_node(self, node_id):
        for n in self.nodes:
            if n.node_id == node_id:
                return n
        raise ValueError(
            f"No node with node_id={node_id} found in gltesting scheduler, do you need to register it first?"
        )

    def add_invite_codes(self, codes: List[schedpb.InviteCode]):
        for code in codes:
            self.invite_codes.append(code)

    async def GetChallenge(
        self,
        req,
    ):
        challenge = Challenge(
            node_id=req.node_id,
            scope=req.scope,
            challenge=bytes([self.next_challenge] * 32),
            used=False,
        )
        self.next_challenge = (self.next_challenge + 1) % 256
        self.challenges.append(challenge)
        return schedpb.ChallengeResponse(challenge=challenge.challenge)

    async def Register(
        self, req: schedpb.RegistrationRequest
    ) -> schedpb.RegistrationResponse:
        self.received_invite_code = req.invite_code

        challenge = None
        for c in self.challenges:
            if c.challenge == req.challenge and not c.used:
                challenge = c
                break
        assert challenge is not None
        assert challenge.node_id == req.node_id
        assert challenge.scope == schedpb.ChallengeScope.REGISTER
        # TODO Verify that the response matches the challenge.

        # Check that we don't already have this node registered:
        if len([n for n in self.nodes if n.node_id == req.node_id]) > 0:
            raise ValueError(
                "could not register the node with the DB, does the node already exist?"
            )

        num = len(self.nodes)
        hex_node_id = challenge.node_id.hex()
        certs.genca(f"/users/{hex_node_id}")

        # Check if the request contains a csr and use it to generate the
        # certificate. Use the old flow if csr is not present.
        if req.csr is not None:
            device_cert = certs.gencert_from_csr(req.csr)
        else:
            device_cert = certs.gencert(f"/users/{hex_node_id}/device")
        node_cert = certs.gencert(f"/users/{hex_node_id}/node")

        directory = self.node_directory / f"node-{num}"
        directory.mkdir(parents=True)

        vstr = req.signer_proto if req.signer_proto is not None else "v0.10.1"
        sv = SignerVersion(name=vstr)

        # Convert the StartupMessage from scheduler.proto to
        # greenlight.proto, yeah, it's suboptimal, and we should merge
        # the two.
        # TODO Remove this conversion once we merged or shared the implementation

        startupmsgs = [
            greenlightpb.StartupMessage(request=sm.request, response=sm.response)
            for sm in req.startupmsgs
        ]

        self.nodes.append(
            Node(
                node_id=req.node_id,
                signer_version=sv,
                initmsg=req.init_msg,
                network=req.network,
                directory=directory,
                identity=node_cert,
                process=None,
                plugin_grpc_uri=None,
                condition=Condition(),
                startupmsgs=startupmsgs,
            )
        )

        crt = device_cert
        key = None

        # No csr was provided, we need to get the cert and the key from the
        # identity.
        if req.csr is None:
            crt = device_cert.cert_chain
            key = device_cert.private_key

        return schedpb.RegistrationResponse(
            device_cert=crt,
            device_key=key,
        )

    async def Recover(self, req):
        challenge = None
        for c in self.challenges:
            if c.challenge == req.challenge and not c.used:
                challenge = c
                break
        assert challenge is not None
        assert challenge.node_id == req.node_id
        assert challenge.scope == schedpb.ChallengeScope.RECOVER
        # TODO Verify that the response matches the challenge.
        hex_node_id = challenge.node_id.hex()

        # Check if the request contains a csr and use it to generate the
        # certificate. Use the old flow if csr is not present.
        if req.csr is not None:
            device_cert = certs.gencert_from_csr(req.csr, recover=True)
            device_key = None
        else:
            device_id = certs.gencert(
                f"/users/{hex_node_id}/recover-{len(self.challenges)}"
            )
            device_key = device_id.private_key
            device_cert = device_id.cert_chain

        return schedpb.RecoveryResponse(device_cert=device_cert, device_key=device_key)

    async def Schedule(self, req):
        n = self.get_node(req.node_id)

        # If already running we just return the existing binding
        if n.process:
            return schedpb.NodeInfoResponse(
                node_id=n.node_id,
                grpc_uri=n.process.grpc_uri,
            )

        node_version = n.signer_version.get_node_version()
        node_version = self.versions.get(node_version, None)

        logging.debug(
            f"Determined that we need to start node_version={node_version} for n.signer_version={n.signer_version}"
        )

        if node_version is None:
            raise ValueError(
                f"No node_version found for n.signer_version={n.signer_version}"
            )

        # Otherwise we need to start a new process
        n.process = NodeProcess(
            node_id=req.node_id,
            init_msg=n.initmsg,
            directory=n.directory,
            network=n.network,
            identity=n.identity,
            version=node_version,
            bitcoind=self.bitcoind,
            startupmsgs=n.startupmsgs,
        )
        n.process.write_node_config(n.network)
        n.process.start()

        with n.condition:
            n.condition.notify_all()

        # Wait for the grpc port to be accessible
        start_time = time.perf_counter()
        timeout = 10
        while True:
            try:
                with socket.create_connection(
                    ("localhost", n.process.grpc_port), timeout=0.1
                ):
                    break
            except Exception:
                time.sleep(0.01)
                if time.perf_counter() - start_time >= timeout:
                    raise TimeoutError(
                        f"Waited too for port localhost:{n.process.grpc_port} to become reachable"
                    )
        # TODO Actually wait for the port to be accessible
        time.sleep(1)

        return schedpb.NodeInfoResponse(
            node_id=n.node_id,
            grpc_uri=n.process.grpc_uri,
        )

    async def MaybeUpgrade(self, req):
        # Very roundabout way of extracting the x509 common name from
        # which we can extract the node_id
        # TODO Implement version ordering and upgrade here
        # common_name = ctx.auth_context()['x509_common_name'][0].decode('ASCII')
        # node_id = common_name.split('/')[2]
        # node = self.get_node(unhexlify(node_id))

        return schedpb.UpgradeResponse(
            old_version="v0.11.0.1",
        )

    async def GetNodeInfo(self, req):
        node = self.get_node(req.node_id)

        if req.wait:
            with node.condition:
                while node.process is not None and node.process.proc is None:
                    logging.info(
                        f"Signer waiting for node {node.node_id.hex()} to get scheduled"
                    )
                    node.condition.wait()

        return schedpb.NodeInfoResponse(
            node_id=req.node_id,
            grpc_uri=node.process.grpc_uri if node.process else None,
        )

    async def ListInviteCodes(self, req) -> schedpb.ListInviteCodesResponse:
        codes = [schedpb.InviteCode(**c) for c in self.invite_codes]
        return schedpb.ListInviteCodesResponse(invite_code_list=codes)

    async def add_outgoing_webhook(self, req) -> schedpb.AddOutgoingWebhookResponse:
        n = self.get_node(req.node_id)
        secret = generate_secret()
        id = self.next_webhook_id
        
        self.webhooks.append({
            "id": id,
            "node_id": n.node_id,
            "uri": req.uri,
            "secret": secret
        })
        
        self.next_webhook_id = self.next_webhook_id + 1
        return schedpb.AddOutgoingWebhookResponse(id=id, secret=secret)
    
    async def list_outgoing_webhooks(self, req) -> schedpb.ListOutgoingWebhooksResponse:
        n = self.get_node(req.node_id)
        webhooks = [schedpb.Webhook(**{"id": w["id"], "uri": w["uri"]}) for w in self.webhooks if w["node_id"] == n.node_id]
        return schedpb.ListOutgoingWebhooksResponse(outgoing_webhooks=webhooks)
    
    async def delete_outgoing_webhooks(self, req) -> greenlightpb.Empty:
        n = self.get_node(req.node_id)
        self.webhooks = [w for w in self.webhooks if not (w["id"] in req.ids and w["node_id"] == n.node_id)]
        return greenlightpb.Empty()
    
    async def rotate_outgoing_webhook_secret(self, req) -> schedpb.WebhookSecretResponse:
        n = self.get_node(req.node_id)
        webhook = next((w for w in self.webhooks if w["id"] == req.webhook_id and w["node_id"] == n.node_id), None)
        
        if webhook is None:
            raise ValueError(
                f"No webhook with id={webhook_id} found in gltesting scheduler"
            )
        
        secret = generate_secret()
        webhook["secret"] = secret
        return schedpb.WebhookSecretResponse(secret=secret)
    

class DebugServicer(schedgrpc.DebugServicer):
    """Collects and analyzes rejected signer requests."""
    
    def __init__(self):
        self.reports: List[schedpb.SignerRejection] = []
    
    async def ReportSignerRejection(self, report):
        self.reports.append(report)
        return greenlightpb.Empty()

class PairingServicer(schedgrpc.PairingServicer):
    """Mocks a pairing backend for local testing"""
    def __init__(self):
        self.sessions: Dict[int, Dict[str, str | bytes]] = {}
        
    async def PairDevice(self, req: schedpb.PairDeviceRequest):
        data = {
            "csr": req.csr,
            "device_name": req.device_name,
            "description": req.description,
            "restrictions": req.restrictions,
        }
        self.sessions[req.session_id] = data
        
        device_cert = certs.gencert_from_csr(req.csr, recover=False, pairing=True)
        return schedpb.PairDeviceResponse(
            session_id=req.session_id,
            device_cert=device_cert)    

Scheduler = AsyncScheduler
