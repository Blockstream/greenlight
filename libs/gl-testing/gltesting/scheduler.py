from glclient import scheduler_pb2_grpc as schedgrpc
from glclient import scheduler_pb2 as schedpb
from concurrent.futures import ThreadPoolExecutor
import grpc
import logging
from collections import namedtuple
import tempfile
from pathlib import Path
from gltesting import certs
from gltesting.node import MockNode
from ephemeral_port_reserve import reserve
import time
from dataclasses import dataclass
from gltesting.identity import Identity
from enum import Enum
import os
import subprocess
import shutil
from typing import Optional
from gltesting.utils import NodeVersion, SignerVersion, Network

@dataclass
class Node:
    node_id: bytes
    signer_version: SignerVersion
    directory: Path
    network: Network
    initmsg: bytes
    identity: Identity
    process: Optional[subprocess.Popen]
    plugin_grpc_uri: Optional[str]


@dataclass
class Challenge:
    node_id: bytes
    challenge: bytes
    scope: str
    used: bool


def enumerate_cln_versions():
    """Search `$PATH` and `$CLN_PATH` for CLN versions."""
    path = os.environ["PATH"].split(":")
    path += os.environ.get("CLN_PATH", "").split(":")
    path = [p for p in path if p != ""]
    version_paths = [shutil.which("lightningd", path=p) for p in path]
    version_paths = [p for p in version_paths if p is not None]

    versions = {}
    for v in version_paths:
        vs = subprocess.check_output([v, "--version"]).strip().decode("ASCII")
        versions[vs] = NodeVersion(path=v, name=vs)

    logging.info(f"Found {len(versions)} versions: {versions}")
    return versions


class Scheduler(object):
    """A mock Scheduler simulating Greenlight's behavior."""

    def __init__(self, grpc_port=2601, identity=None, node_directory=None):
        self.grpc_port = grpc_port
        self.server = None
        print("Starting scheduler with caroot", identity.caroot)
        self.identity = identity
        self.challenges: List[Challenge] = []
        self.next_challenge: int = 1
        self.nodes: List[Node] = []
        self.versions = enumerate_cln_versions()

        if node_directory is not None:
            self.node_directory = node_directory
        else:
            self.node_directory = Path(tempfile.mkdtemp())

    @property
    def grpc_addr(self):
        return f"https://localhost:{self.grpc_port}"

    def start(self):
        logging.info(f"Starting scheduler on port {self.grpc_port}")
        if self.server is not None:
            raise ValueError("Server already running")

        cred = self.identity.to_server_credentials()
        self.server = grpc.server(ThreadPoolExecutor(max_workers=10))
        schedgrpc.add_SchedulerServicer_to_server(self, self.server)
        self.server.add_secure_port(f"[::]:{self.grpc_port}", cred)
        self.server.start()
        logging.info(f"Scheduler started on port {self.grpc_port}")

    def stop(self):
        self.server.stop(grace=1)
        self.server = None

    def get_node(self, node_id):
        for n in self.nodes:
            if n.node_id == node_id:
                return n
        raise ValueError(
            f"No node with node_id={node_id} found in gltesting scheduler, do you need to register it first?"
        )

    def Register(self, req, ctx):
        challenge = None
        for c in self.challenges:
            if c.challenge == req.challenge and not c.used:
                challenge = c
                break
        assert challenge is not None
        assert challenge.node_id == req.node_id
        assert challenge.scope == schedpb.ChallengeScope.REGISTER
        # TODO Verify that the response matches the challenge.

        num = len(self.nodes)
        hex_node_id = challenge.node_id.hex()
        certs.genca(f"/users/{hex_node_id}")

        device_id = certs.gencert(f"/users/{hex_node_id}/device")
        node_cert = certs.gencert(f"/users/{hex_node_id}/node")

        directory = self.node_directory / f"node-{num}"
        directory.mkdir(parents=True)
        self.nodes.append(
            Node(
                node_id=req.node_id,
                signer_version=req.signer_proto
                if req.signer_proto is not None
                else "v0.10.1",
                initmsg=req.init_msg,
                network=req.network,
                directory=directory,
                identity=device_id,
                process=None,
                plugin_grpc_uri=None,
            )
        )

        return schedpb.RegistrationResponse(
            device_cert=device_id.cert_chain,
            device_key=device_id.private_key,
        )

    def Recover(self, req, ctx):
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
        device_id = certs.gencert(
            f"/users/{hex_node_id}/recover-{len(self.challenges)}"
        )
        return schedpb.RecoveryResponse(
            device_cert=device_id.cert_chain,
            device_key=device_id.private_key,
        )

    def GetChallenge(self, req, ctx):
        challenge = Challenge(
            node_id=req.node_id,
            scope=req.scope,
            challenge=bytes([self.next_challenge] * 32),
            used=False,
        )
        self.next_challenge = (self.next_challenge + 1) % 256

        self.challenges.append(challenge)

        return schedpb.ChallengeResponse(challenge=challenge.challenge)

    def Schedule(self, req, ctx):
        node = self.get_node(req.node_id)

        if node.instance.server is None:
            node.instance.start()

        time.sleep(0.1)
        return schedpb.NodeInfoResponse(
            node_id=node.node_id,
            grpc_uri=node.instance.grpc_addr,
        )

    def GetNodeInfo(self, req, ctx):
        node = self.get_node(req.node_id)
        print(node)
        return schedpb.NodeInfoResponse(node_id=req.node_id)

    def MaybeUpgrade(self, req, ctx):
        # TODO extract node_id from the request
        ident = ctx.peer_identities()
        raise NotImplementedError("Method not implemented!")
