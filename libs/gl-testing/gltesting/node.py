from pathlib import Path
from gltesting.identity import Identity
from gltesting.utils import NodeVersion
from binascii import hexlify
from glclient import greenlight_pb2_grpc as nodegrpc
from glclient import greenlight_pb2 as nodepb
import logging
import grpc
from concurrent.futures import ThreadPoolExecutor
import subprocess
from gltesting.utils import NodeVersion, Network
from pyln.testing.utils import TailableProc, BitcoinD
from ephemeral_port_reserve import reserve
import os


target_dir = Path(
    os.environ.get(
        'CARGO_TARGET_DIR',
        Path(__file__).parent / ".." / ".." / "target"
    )
)

bin_dir = target_dir / os.environ.get('RUST_PROFILE', 'debug')
plugin_path = bin_dir / "gl-plugin"
signerproxy_path = bin_dir / "gl-signerproxy"


class NodeProcess(TailableProc):
    """A node running under the control of a scheduler.

    Clients can control it over the grpc plugin, and signers can
    attach to provide signatures when required.
    """

    def __init__(
            self,
            node_id: bytes,
            init_msg: bytes,
            directory: Path,
            network: Network,
            identity: Identity,
            version: NodeVersion,
            bitcoind: BitcoinD,
    ):
        TailableProc.__init__(self, str(directory), verbose=True)
        self.identity = identity
        self.version  = version
        self.proc: Optional[subprocess.Popen] = None
        self.directory = directory
        self.node_id = node_id
        self.init_msg = init_msg
        self.executable = self.version.path
        self.bind: Optional[str] = None
        self.grpc_uri: Optional[str] = None
        self.network = network
        self.bitcoind = bitcoind
        self.prefix = "node"

        # Stage the identity so the plugin can pick it up.
        cert_path = self.directory / "certs" / "users" / "1"
        cert_path.mkdir(parents=True, exist_ok=True)

        with (self.directory / "certs" / "ca.pem").open("wb") as f:
            f.write(identity.caroot)

        with (cert_path / "server-key.pem").open("wb") as f:
            f.write(identity.private_key)

        with (cert_path / "server.crt").open("wb") as f:
            f.write(identity.cert_chain)
        self.p2p_port = reserve()

        self.cmd_line = [
            str(self.executable),
            f'--lightning-dir={self.directory}',
            f'--network={network}',
            '--log-level=debug',
            '--bitcoin-rpcuser=rpcuser',
            '--bitcoin-rpcpassword=rpcpass',
            f'--bitcoin-rpcconnect=localhost:{self.bitcoind.rpcport}',
            # The following will allow us to support the `stream_log`
            #RPC method after updating to pyln-testing==0.12
            #f'--log-file=log',
            #f'--log-file=-',
            '--rescan=1',
            "--log-timestamps=false",
            "--cltv-final=6",
            f"--addr=127.0.0.1:{self.p2p_port}",
            # Stock `cln-grpc` disabled in favor of `gl-plugin`
            '--disable-plugin=cln-grpc',
            f'--subdaemon=hsmd:{signerproxy_path}',
            f'--important-plugin={plugin_path}',
            '--dev-bitcoind-poll=5',
            '--dev-fast-gossip',
        ]

    def start(self):
        path = os.environ.get('PATH')
        # Need to add the subdaemon directory to PATH so the
        # signerproxy can find the version.
        libexec_path = self.executable.parent / '..' / 'libexec' / 'c-lightning'

        self.grpc_port = reserve()
        self.bind = f"127.0.0.1:{self.grpc_port}"
        self.grpc_uri = f"https://localhost:{self.grpc_port}"
        self.env.update({
            "GL_CERT_PATH": self.directory / "certs",
            "PATH": f"{self.version.path}:{libexec_path}:{path}",
            "CLN_VERSION": self.version.name,
            "GL_NODE_NETWORK": self.network,
            "GL_NODE_ID": self.node_id.hex(),
            "GL_NODE_INIT": self.init_msg.hex(),
            "GL_NODE_BIND": self.bind,
            "GL_PLUGIN_CLIENTCA_PATH": str(self.directory / "certs" / "ca.pem"),
            "RUST_LOG": os.environ.get(
                "RUST_LOG",
                "gl_client=trace,gl_signerproxy=trace,gl_plugin=trace,cln_plugin=trace,cln_rpc=trace,cln_grpc=trace,info"
            ),
        })

        TailableProc.start(self)
        self.wait_for_log("Server started with public key")
