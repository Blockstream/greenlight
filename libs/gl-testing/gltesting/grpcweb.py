# A simple grpc-web proxy enabling web-clients to talk to
# Greenlight. Unlike the direct grpc interface exposed by the node and
# the node domain proxy, the grpc-web proxy does not require a client
# certificate from the client, making it possible for browsers to talk
# to it. The client authentication via client certificates is no
# longer present, but the payloads are still signed by the authorized
# client, assuring authentication of the client.

from gltesting.scheduler import Scheduler
from ephemeral_port_reserve import reserve
from threading import Thread, Event
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler
import logging
import struct


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

    def start(self):
        self._thread = Thread(target=self.run, daemon=True)
        self.logger.info(f"Starting grpc-web-proxy on port {self.web_port}")
        self.running = True
        server_address = ("127.0.0.1", self.web_port)

        self.httpd = ThreadingHTTPServer(server_address, Handler)
        self.httpd.grpc_port = self.grpc_port
        self.logger.debug(f"Server startup complete")
        self._thread.start()

    def run(self):
        self.httpd.serve_forever()

    def stop(self):
        self.logger.info(f"Stopping grpc-web-proxy running on port {self.web_port}")
        self.httpd.shutdown()
        self._thread.join()


class Handler(BaseHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        self.logger = logging.getLogger("gltesting.grpcweb.Handler")
        BaseHTTPRequestHandler.__init__(self, *args, **kwargs)

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

        # TODO extract the `glauthpubkey` and the `glauthsig`, then
        # verify them. Fail the call if the verification fails,
        # forward otherwise.
        # This is just a test server, and we don't make use of the
        # multiplexing support in `h2`, which simplifies this proxy
        # quite a bit. The production server maintains a cache of
        # connections and multiplexes correctly.

        import httpx

        url = f"http://localhost:{self.server.grpc_port}{self.path}"
        self.logger.debug(f"Forwarding request to '{url}'")
        headers = {
            "te": "trailers",
            "Content-Type": "application/grpc",
            "grpc-accept-encoding": "idenity",
            "user-agent": "My bloody hacked up script",
        }
        content = struct.pack("!cI", flags, length) + body
        req = httpx.Request(
            "POST",
            url,
            headers=headers,
            content=content,
        )
        client = httpx.Client(http1=False, http2=True)

        res = client.send(req)
        res = client.send(req)

        canned = b"\n\rheklllo world"
        l = struct.pack("!I", len(canned))
        self.wfile.write(b"HTTP/1.0 200 OK\n\n")
        self.wfile.write(b"\x00")
        self.wfile.write(l)
        self.wfile.write(canned)
        self.wfile.flush()
