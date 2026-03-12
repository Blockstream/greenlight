#!/usr/bin/env python3
import os, signal, sys, tempfile, logging, json, threading, time
from pathlib import Path
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from concurrent.futures import ThreadPoolExecutor
from ephemeral_port_reserve import reserve
from gltesting import certs
from gltesting.scheduler import Scheduler
from gltesting.network import GlNodeFactory
from gltesting.clients import Clients
from gltesting import get_plugins_dir
from pyln.testing.utils import BitcoinD, wait_for
from pyln.testing.db import SqliteDbProvider
from pyln import grpc as clnpb

logging.basicConfig(level=logging.WARNING)

ENV_FILE = Path('/tmp/gltests/.env')
TMP_DIR  = Path(tempfile.mkdtemp(prefix='gl-lsp-setup-'))

def main():
    # 1. Certs
    cert_dir = TMP_DIR / 'certs'
    cert_dir.mkdir(parents=True)
    os.environ['GL_CERT_PATH'] = str(cert_dir)
    os.environ['GL_CA_CRT']    = str(cert_dir / 'ca.pem')

    certs.genca('/')
    certs.genca('/services')
    certs.gencert('/services/scheduler')
    certs.genca('/users')
    nobody_id    = certs.gencert('/users/nobody')
    scheduler_id = certs.gencert('/services/scheduler')

    os.environ['GL_NOBODY_CRT'] = str(nobody_id.cert_chain_path)
    os.environ['GL_NOBODY_KEY'] = str(nobody_id.private_key_path)

    # 2. Bitcoind
    btc_dir = TMP_DIR / 'bitcoind'
    btc_dir.mkdir()
    bitcoind = BitcoinD(bitcoin_dir=str(btc_dir))
    bitcoind.start()
    bitcoind.generate_block(105)

    # 3. Scheduler
    grpc_port = reserve()
    s = Scheduler(bitcoind=bitcoind.get_proxy(), grpc_port=grpc_port, identity=scheduler_id)
    os.environ['GL_SCHEDULER_GRPC_URI'] = s.grpc_addr
    s.start()

    # 4. LSP node
    nodes_dir = TMP_DIR / 'nodes'
    nodes_dir.mkdir()
    executor    = ThreadPoolExecutor(max_workers=8)
    db_provider = SqliteDbProvider(str(nodes_dir))

    from pyln.testing.utils import LightningNode
    from unittest.mock import MagicMock
    request = MagicMock()
    request.node.get_closest_marker.return_value = None

    nf = GlNodeFactory(
        request=request,
        testname='test_setup',
        bitcoind=bitcoind,
        executor=executor,
        directory=str(nodes_dir),
        db_provider=db_provider,
        node_cls=LightningNode,
        jsonschemas={},
    )

    policy_plugin = get_plugins_dir() / 'lsps2_policy.py'
    lsp = nf.get_node(options={
        'experimental-lsps2-service': None,
        'experimental-lsps2-promise-secret': 'A' * 64,
        'important-plugin': str(policy_plugin),
    })

    lsp_node_id   = lsp.info['id']
    lsp_node_host = '127.0.0.1'
    lsp_node_port = lsp.daemon.port
    lsp.fundwallet(sats=2 * 10**6)

    # 5. Clients (for /new-lsp-node endpoint)
    clients_dir = TMP_DIR / 'clients'
    clients_dir.mkdir()
    clients = Clients(
        directory=clients_dir,
        scheduler=s,
        nobody_id=nobody_id,
    )

    # 6. HTTP server
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *args): pass

        def _send_json(self, status: int, payload: dict):
            body = json.dumps(payload).encode()
            self.send_response(status)
            self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            self.wfile.flush()

        def _read_body(self) -> dict:
            length = int(self.headers.get('Content-Length', 0))
            return json.loads(self.rfile.read(length)) if length else {}

        def do_POST(self):
            try:
                if self.path == '/connect-to-lsp':
                    self._handle_connect_to_lsp()
                elif self.path == '/fund-wallet':
                    self._handle_fund_wallet()
                elif self.path == '/lsp-invoice':
                    self._handle_lsp_invoice()
                elif self.path == '/btc-address':
                    self._handle_btc_address()
                else:
                    self._send_json(404, {'error': f'Unknown path: {self.path}'})
            except Exception as e:
                self._send_json(500, {'error': str(e)})

        def _handle_fund_wallet(self):
            body    = self._read_body()
            address = body['address']
            amount  = body.get('amount', 100_000_000)  # sats
            nf.bitcoind.rpc.sendtoaddress(address, amount / 100_000_000)
            nf.bitcoind.generate_block(6)
            self._send_json(200, {'ok': True})

        def _handle_connect_to_lsp(self):
            body = self._read_body()
            secret = bytes.fromhex(body['secret'])
            client = clients.new(secret=secret[:32])
            client.register(configure=True)
            signer = client.signer()
            handle = signer.run_in_thread()
            try:
                gl_node = client.node()
                gl_node.connect_peer(lsp_node_id, f'{lsp_node_host}:{lsp_node_port}')
            finally:
                handle.shutdown()
            creds_path = client.directory / 'greenlight.auth'
            self._send_json(200, {'creds_path': str(creds_path)})

        def _handle_lsp_invoice(self):
            body = self._read_body()
            import pyln.client
            lsp_rpc = pyln.client.LightningRpc(lsp.rpc.socket_path)
            invoice = lsp_rpc.invoice(
                amount_msat=body.get('amount_msat', 0),
                label=body.get('label', f'test-{int(time.time())}'),
                description=body.get('description', 'Test payment'),
            )
            self._send_json(200, {'bolt11': invoice['bolt11']})

        def _handle_btc_address(self):
            address = nf.bitcoind.rpc.getnewaddress()
            self._send_json(200, {'address': address})

    http = ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    http_port = http.server_address[1]
    threading.Thread(target=http.serve_forever, daemon=True).start()

    # 7. Write env for Jest
    ENV_FILE.parent.mkdir(parents=True, exist_ok=True)
    ENV_FILE.write_text('\n'.join([
        f"GL_SCHEDULER_GRPC_URI={s.grpc_addr}",
        f"GL_CA_CRT={cert_dir / 'ca.pem'}",
        f"GL_NOBODY_CRT={nobody_id.cert_chain_path}",
        f"GL_NOBODY_KEY={nobody_id.private_key_path}",
        f"LSP_RPC_SOCKET={lsp.rpc.socket_path}",
        f"LSP_NODE_ID={lsp_node_id}",
        f"TEST_SETUP_SERVER_URL=http://127.0.0.1:{http_port}",
    ]))
    print('✅ LSP setup ready', flush=True)

    # 8. Block until teardown
    def shutdown(*_):
        nf.killall([])
        s.stop()
        bitcoind.stop()
        executor.shutdown(wait=False)
        sys.exit(0)

    signal.signal(signal.SIGTERM, shutdown)
    signal.pause()

if __name__ == '__main__':
    main()
