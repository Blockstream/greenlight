#!/usr/bin/env python3
import os, signal, sys, tempfile, logging, json, threading
from pathlib import Path
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
from concurrent.futures import ThreadPoolExecutor
from ephemeral_port_reserve import reserve
from gltesting import certs
from gltesting.scheduler import Scheduler
from gltesting.network import GlNodeFactory
from gltesting import get_plugins_dir
from pyln.testing.utils import BitcoinD
from pyln.testing.db import SqliteDbProvider

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
    nobody_id     = certs.gencert('/users/nobody')
    scheduler_id  = certs.gencert('/services/scheduler')

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

    # 4. LSP node via GlNodeFactory
    nodes_dir = TMP_DIR / 'nodes'
    nodes_dir.mkdir()
    executor = ThreadPoolExecutor(max_workers=8)
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

    # 5. HTTP funding server
    class FundHandler(BaseHTTPRequestHandler):
        def log_message(self, *args): pass

        def do_POST(self):
            try:
                body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
                address = body['address']
                amount  = body.get('amount', 100_000_000)
                
                # Send to address (convert sats to BTC)
                nf.bitcoind.rpc.sendtoaddress(address, amount / 100_000_000)
                nf.bitcoind.generate_block(6)
                
                response = b'{"ok":true}'
                self.send_response(200)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(response)))
                self.end_headers()
                self.wfile.write(response)
                self.wfile.flush()
            except Exception as e:
                error = json.dumps({'error': str(e)}).encode()
                self.send_response(500)
                self.send_header('Content-Type', 'application/json')
                self.send_header('Content-Length', str(len(error)))
                self.end_headers()
                self.wfile.write(error)
                self.wfile.flush()
    
    http = ThreadingHTTPServer(('127.0.0.1', 0), FundHandler)
    port = http.server_address[1]
    threading.Thread(target=http.serve_forever, daemon=True).start()

    # 6. Write env for Jest
    ENV_FILE.write_text('\n'.join([
        f"GL_SCHEDULER_GRPC_URI={s.grpc_addr}",
        f"GL_CA_CRT={cert_dir / 'ca.pem'}",
        f"GL_NOBODY_CRT={nobody_id.cert_chain_path}",
        f"GL_NOBODY_KEY={nobody_id.private_key_path}",
        f"LSP_RPC_SOCKET={lsp.rpc.socket_path}",
        f"LSP_NODE_ID={lsp.info['id']}",
        f"LSP_PROMISE_SECRET={'A' * 64}",
        f"GL_FUND_SERVER=http://127.0.0.1:{port}",
    ]))
    print('✅ LSP setup ready', flush=True)

    # 7. Block until Jest teardown sends SIGTERM
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
