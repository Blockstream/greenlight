# A simple grpc-web proxy enabling web-clients to talk to
# Greenlight. Unlike the direct grpc interface exposed by the node and
# the node domain proxy, the grpc-web proxy does not require a client
# certificate from the client, making it possible for browsers to talk
# to it. The client authentication via client certificates is no
# longer present, but the payloads are still signed by the authorized
# client, assuring authentication of the client.

from gltesting.scheduler import Scheduler
from ephemeral_port_reserve import reserve
from threading import Thread
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
import logging
import struct
import httpx
from dataclasses import dataclass
from typing import Dict

class GrpcWebProxy(object):
    def __init__(self, scheduler: Scheduler, grpc_port: int):
        self.logger = logging.getLogger("gltesting.grpcweb.GrpcWebProxy")
        self.scheduler = scheduler
        self.web_port = reserve()
        self._thread: None | Thread = None
        self.running = False
        self.grpc_port = grpc_port
        self.httpd: None | ThreadingHTTPServer = None
        self.logger.info(
            f"GrpcWebProxy configured to forward requests from web_port={self.web_port} to grpc_port={self.grpc_port}"
        )
        self.handler_cls = Handler

    def start(self):
        self._thread = Thread(target=self.run, daemon=True)
        self.logger.info(f"Starting grpc-web-proxy on port {self.web_port}")
        self.running = True
        server_address = ("127.0.0.1", self.web_port)

        self.httpd = ThreadingHTTPServer(server_address, self.handler_cls)
        self.httpd.grpc_port = self.grpc_port

        # Just a simple way to pass the scheduler to the handler
        self.httpd.scheduler = self.scheduler

        self.logger.debug(f"Server startup complete")
        self._thread.start()

    def run(self):
        self.httpd.serve_forever()

    def stop(self):
        self.logger.info(f"Stopping grpc-web-proxy running on port {self.web_port}")
        self.httpd.shutdown()
        self._thread.join()


@dataclass
class Request:
    body: bytes
    headers: Dict[str, str]
    flags: int
    length: int


@dataclass
class Response:
    body: bytes
    headers: Dict[str, str]


class Handler(BaseHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.logger = logging.getLogger("gltesting.grpcweb.Handler")
        BaseHTTPRequestHandler.__init__(self, *args, **kwargs)

    def proxy(self, request) -> Response:
        """Callback called with the request, implementing the proxying."""
        url = f"http://localhost:{self.server.grpc_port}{self.path}"
        self.logger.debug(f"Forwarding request to '{url}'")
        headers = {
            "te": "trailers",
            "Content-Type": "application/grpc",
            "grpc-accept-encoding": "identity",
            "user-agent": "gl-testing-grpc-web-proxy",
        }
        content = struct.pack("!cI", request.flags, request.length) + request.body
        req = httpx.Request(
            "POST",
            url,
            headers=headers,
            content=content,
        )
        timeout = httpx.Timeout(10.0, connect=5.0)
        client = httpx.Client(http1=False, http2=True, timeout=timeout)
        res = client.send(req)
        return Response(body=res.content, headers=res.headers)

    def auth(self, request: Request) -> bool:
        """Authenticate the request. True means allow."""
        return True

    def do_POST(self):
        # We don't actually touch the payload, so we do not really
        # care about the flags ourselves. The upstream sysmte will
        # though.
        flags = self.rfile.read(1)

        # We have the length from above already, but that includes the
        # header. Ensure that the two values match up.
        strlen = self.rfile.read(4)
        (length,) = struct.unpack_from("!I", strlen)
        l = int(self.headers.get("Content-Length"))
        assert l == length + 5

        # Now we can finally read the body, It is kept as is, so no
        # need to decode it, and we can treat it as opaque blob.
        body = self.rfile.read(length)

        req = Request(body=body, headers=self.headers, flags=flags, length=length)
        if not self.auth(req):
            self.send_response(401)
            self.send_header("Content-Type", "application/grpc")
            self.send_header("grpc-status", "16")
            self.end_headers()
            self.wfile.write(b"Unauthorized")
            return
        
        response = self.proxy(req)
        self.send_response(200)
        self.send_header("Content-Type", "application/grpc")
        self.send_header("grpc-status", response.headers.get("grpc-status", "0"))
        self.end_headers()
        self.wfile.write(response.body)
        self.wfile.flush()


class NodeHandler(Handler):
    """A handler that is aware of nodes, their auth and how they schedule."""

    def __init__(self, *args, **kwargs):
        self.logger = logging.getLogger("gltesting.grpcweb.NodeHandler")
        BaseHTTPRequestHandler.__init__(self, *args, **kwargs)

    def auth(self, request: Request) -> bool:
        # TODO extract the `glauthpubkey` and the `glauthsig`, then
        # verify them. Fail the call if the verification fails,
        # forward otherwise.
        # This is just a test server, and we don't make use of the
        # multiplexing support in `h2`, which simplifies this proxy
        # quite a bit. The production server maintains a cache of
        # connections and multiplexes correctly.
        pk = request.headers.get("glauthpubkey", None)
        sig = request.headers.get("glauthsig", None)
        ts = request.headers.get("glts", None)

        if not pk:
            self.logger.warning(f"Missing public key header: headers={request.headers}")
            return False

        if not sig:
            self.logger.warning(f"Missing signature header: headers={request.headers}")
            return False

        if not ts:
            self.logger.warning(f"Missing timestamp header: headers={request.headers}")
            return False

        # TODO Check the signature.
        return True

    def proxy(self, request: Request):
        # Fetch current location of the node

        pk = request.headers.get("glauthpubkey")
        from base64 import b64decode

        pk = b64decode(pk)

        node = self.server.scheduler.get_node(pk)
        self.logger.debug(f"Found node for node_id={pk.hex()}")

        # TODO Schedule node if not scheduled

        client_cert = node.identity.private_key
        ca_path = node.identity.caroot_path

        # Load TLS client cert info client
        ctx = httpx.create_ssl_context(
            verify=ca_path,
            cert=(
                node.identity.cert_chain_path,
                node.identity.private_key_path,
            ),
        )
        client = httpx.Client(http1=False, http2=True, verify=ctx)

        url = f"{node.process.grpc_uri}{self.path}"
        headers = {
            "te": "trailers",
            "Content-Type": "application/grpc",
            "grpc-accept-encoding": "identity",
            "user-agent": "gl-testing-grpc-web-proxy",
        }
        content = struct.pack("!cI", request.flags, request.length) + request.body

        # Forward request
        req = httpx.Request(
            "POST",
            url,
            headers=headers,
            content=content,
        )
        
        try:
            self.logger.debug(f"Sending grpc request req={req}, payload={content} to url={url}")
            res = client.send(req)
            # Capture the error from header and send it in the body as well
            status = res.headers.get("grpc-status", "0")
            logging.debug(f"Server returned grpc-status={status}")
            if status != "0":
                error_message = res.headers.get("grpc-message", "None")
                self.logger.error(f"Received grpc-status={status}, error={error_message}")
                self.logger.warning(f"gRPC status code received: {status}")
                self.logger.warning(f"gRPC message received: {error_message}")
                error = error_message.encode("utf-8")
                error_res = struct.pack("!cI", request.flags, len(error)) + error
                return Response(body=error_res, headers=res.headers)
            # Return successful response
            return Response(body=res.content, headers=res.headers)
        except Exception as e:
            self.logger.warning(f"gRPC request error received: {str(e)}")
            error_message = f"Internal Server Error: {str(e)}"
            error = error_message.encode("utf-8")
            error_res = struct.pack("!cI", request.flags, len(error)) + error
            return Response(body=error_res, headers={"Content-Type": "application/grpc", "grpc-status": "13"})
