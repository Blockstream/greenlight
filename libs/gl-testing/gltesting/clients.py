"""Utilities to interact with the scheduler or the nodes.
"""

from gltesting.scheduler import Scheduler
from gltesting.identity import Identity
from pathlib import Path
from typing import Optional
import logging
import glclient
import os

# We only run on regtest!
NETWORK = "regtest"


class Client:
    """A client is any software that interacts with the scheduler or a node.

    Clients have their own identity, and may have an associated signer
    with its seed in the `hsm_secret` file. Depending on whether they
    are associated with a node they'll also have their own identity
    with which to talk with the node adn scheduler.

    """

    def __init__(
        self,
        directory: Path,
        scheduler: Scheduler,
        secret: Optional[bytes] = None,
        name: Optional[str] = None,
    ):
        """Create a new client, with its own directory, identity and name.

        Keyword arguments:
        name -- An optional name to identify the client by in logs
        """
        self.log = logging.getLogger(name if name else "gltesting.clients.Client")
        self.directory = directory
        self.directory.mkdir(parents=True, exist_ok=True)
        self.schedsvc = scheduler

        if secret is not None:
            self.log.debug("Initializing hsm_secret with provided secret")
            assert len(secret) == 32
            with (self.directory / "hsm_secret").open(mode="wb") as f:
                f.write(secret)

        # Use the signer to derive the node_id
        signer = self.signer()
        self.node_id = signer.node_id()
        self.log.info(f"Client for node_id={self.node_id.hex()} initialized")

    def tls(self) -> glclient.TlsConfig:
        """Load the most concrete identity we have available.

        This is either `/users/nobody` if we haven't registered,
        recovered or paired yet, or `users/xyz/device` if we have done
        any of the above.

        """
        capath = self.directory / "ca.crt"
        keypath = self.directory / "device-key.pem"
        if keypath.exists():
            self.log.info(f"Loading client identity from {keypath}")
            certpath = self.directory / "device.crt"
        else:
            certpath = self.directory / "nobody.crt"
            keypath = self.directory / "nobody-key.pem"
            self.log.info(f"Loading generic nobody identity from {keypath}")

        return (
            glclient.TlsConfig()
            .with_ca_certificate(capath.open(mode="r").read())
            .identity(certpath.open(mode="r").read(), keypath.open(mode="r").read())
        )

    def creds(self) -> glclient.Credentials:
        """Load the credentials data

        Returns the devices credentials data
        """
        self.log.info("Trying to find greenlight.auth")
        credspath = self.directory / "greenlight.auth"
        if credspath.exists():
            self.log.info(f"Loading credentials data from {credspath}")
            creds = glclient.Credentials.from_path(str(credspath.absolute()))
            return creds
        else:
            certpath = self.directory / "nobody.crt"
            keypath = self.directory / "nobody-key.pem"
            capath = self.directory / "ca.crt"
            self.log.info(f"Loading generic nobody credentials from {self.directory}")
            creds = glclient.Credentials.nobody_with(
                certpath.open(mode="rb").read(),
                keypath.open(mode="rb").read(),
                capath.open(mode="rb").read(),
            )
            return creds

    def scheduler(self) -> glclient.Scheduler:
        """Return a scheduler stub configured with our identity if configured."""
        return glclient.Scheduler(self.node_id, network=NETWORK, tls=glclient.TlsConfig(self.creds()))

    def signer(self) -> glclient.Signer:
        secret = (self.directory / "hsm_secret").open(mode="rb").read()
        keypath = self.directory / "device-key.pem"
        have_certs = keypath.exists()

        self.directory.mkdir(exist_ok=True)
        signer = glclient.Signer(secret, NETWORK, glclient.TlsConfig(self.creds()))

        return signer

    def node(self):
        return self.scheduler().node(self.creds())

    def register(self, configure: bool = True) -> None:
        """A helper to register and configure the node

        Keyword arguments:
        configure -- Whether or not we should store the certificate in our dir
        """
        r = self.scheduler().register(self.signer())
        if configure:
            with (self.directory / "device.crt").open("w") as f:
                f.write(r.device_cert)
            with (self.directory / "device-key.pem").open("w") as f:
                f.write(r.device_key)
            with (self.directory / "greenlight.auth").open("wb") as f:
                f.write(r.creds)

    def recover(self, configure: bool = True) -> None:
        r = self.scheduler().recover(self.signer())
        if configure:
            with (self.directory / "device.crt").open("w") as f:
                f.write(r.device_cert)
            with (self.directory / "device-key.pem").open("w") as f:
                f.write(r.device_key)
            with (self.directory / "greenlight.auth").open("wb") as f:
                f.write(r.creds)

    def find_node(self):
        """If we registered find the matching node in the scheduler"""
        for n in self.schedsvc.nodes:
            if n.node_id == self.node_id:
                return n
        return None


class Clients:
    """A helper object with utilities to manage clients.

    Create or clone a client.
    """

    def __init__(self, directory: Path, scheduler: Scheduler, nobody_id: Identity):
        self.directory = directory
        self.next_client_id = 1
        self.scheduler = scheduler
        self.nobody_id = nobody_id

    def new(self, secret: Optional[bytes] = None) -> Client:
        id = self.next_client_id
        self.next_client_id += 1
        directory = self.directory / f"client-{id}"
        directory.mkdir(parents=True)

        # Write the nobody id in here, so the client can load it if
        # needed.
        with (directory / "nobody.crt").open(mode="wb") as f:
            f.write(self.nobody_id.cert_chain)

        with (directory / "nobody-key.pem").open(mode="wb") as f:
            f.write(self.nobody_id.private_key)

        with (directory / "ca.crt").open(mode="wb") as f:
            f.write(self.nobody_id.caroot)

        if secret is None:
            secret = bytes([id] * 32)

        logging.debug(f"Creating new client in {directory}")
        c = Client(
            scheduler=self.scheduler,
            directory=directory,
            secret=secret,
            name=f"Client-{id}",
        )
        return c

    def new_keyless(self):
        """Create a new client without a key of its own.

        This is intended to be a constrained client, which cannot
        register or recover on its own and requires another client,
        with key, to authorize it.

        Currently this pairing mechanism is not yet implemented.
        """
        raise NotImplementedError
