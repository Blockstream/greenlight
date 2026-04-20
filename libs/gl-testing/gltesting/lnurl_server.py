# A CLN-backed LNURL server for integration testing.
#
# Implements enough of LUD-01, LUD-03, LUD-06, LUD-09 and LUD-16 to
# exercise gl-client and gl-sdk LNURL flows end-to-end. Backed by a
# real CLN node (via pyln.client) so invoices are real BOLT11s that
# the system-under-test can actually pay / receive.

from ephemeral_port_reserve import reserve
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from threading import Thread
from urllib.parse import urlparse, parse_qs
import hashlib
import json
import logging
import secrets


class LnurlServer:
    """HTTP server that exposes LNURL-pay, LNURL-withdraw and Lightning
    Address endpoints backed by a CLN node.

    Routes:
        GET  /lnurlp                         → LUD-06 payRequest response
        GET  /lnurlp/callback?amount=&comment=
                                             → BOLT11 invoice + optional successAction
        GET  /.well-known/lnurlp/{username}  → LUD-16 (same payRequest)
        GET  /lnurlw                         → LUD-03 withdrawRequest response
        GET  /lnurlw/callback?k1=&pr=        → service pays the invoice
    """

    def __init__(
        self,
        cln_node,
        *,
        domain: str = "127.0.0.1",
        username: str = "alice",
        min_sendable: int = 1_000,
        max_sendable: int = 100_000_000,
        min_withdrawable: int = 1_000,
        max_withdrawable: int = 100_000_000,
        comment_allowed: int = 0,
        success_action: dict | None = None,
    ):
        self.logger = logging.getLogger("gltesting.lnurl_server")
        self.cln_node = cln_node
        self.cln_rpc = cln_node.rpc
        self.domain = domain
        self.username = username
        self.min_sendable = min_sendable
        self.max_sendable = max_sendable
        self.min_withdrawable = min_withdrawable
        self.max_withdrawable = max_withdrawable
        self.comment_allowed = comment_allowed
        self.success_action = success_action

        self.port = reserve()
        self._thread: Thread | None = None
        self._httpd: ThreadingHTTPServer | None = None

        # Metadata for the pay request (LUD-06 mandates text/plain,
        # LUD-16 requires a text/identifier entry for lightning addresses).
        # We include the port in the identifier because the test domain is
        # localhost-based and the lightning address includes the port.
        self.metadata = json.dumps(
            [
                ["text/plain", f"Pay to {username}"],
                ["text/identifier", f"{username}@{domain}:{self.port}"],
            ]
        )

        # Each withdraw session issues a fresh k1 and remembers it until consumed
        self._pending_withdrawals: dict[str, dict] = {}

        # Logs of all incoming callback requests — tests inspect these
        self.pay_callbacks: list[dict] = []
        self.withdraw_callbacks: list[dict] = []

    # ── URLs ──────────────────────────────────────────────────

    @property
    def base_url(self) -> str:
        return f"http://{self.domain}:{self.port}"

    @property
    def pay_url(self) -> str:
        return f"{self.base_url}/lnurlp"

    @property
    def lightning_address(self) -> str:
        return f"{self.username}@{self.domain}:{self.port}"

    @property
    def lightning_address_url(self) -> str:
        return f"{self.base_url}/.well-known/lnurlp/{self.username}"

    @property
    def withdraw_url(self) -> str:
        return f"{self.base_url}/lnurlw"

    # ── Lifecycle ────────────────────────────────────────────

    def start(self):
        server_address = ("127.0.0.1", self.port)
        handler_cls = _handler_factory(self)
        self._httpd = ThreadingHTTPServer(server_address, handler_cls)
        self._thread = Thread(target=self._httpd.serve_forever, daemon=True)
        self._thread.start()
        self.logger.info(f"LnurlServer running on {self.base_url}")

    def stop(self):
        if self._httpd is not None:
            self._httpd.shutdown()
            self._httpd.server_close()
        if self._thread is not None:
            self._thread.join()
        self.logger.info("LnurlServer stopped")

    # ── Handler callbacks (invoked from the HTTP thread) ─────

    def build_pay_response(self, callback_path: str) -> dict:
        return {
            "tag": "payRequest",
            "callback": f"{self.base_url}{callback_path}",
            "minSendable": self.min_sendable,
            "maxSendable": self.max_sendable,
            "metadata": self.metadata,
            "commentAllowed": self.comment_allowed,
        }

    def handle_pay_callback(self, amount_msat: int, comment: str | None) -> dict:
        """Generate an invoice on the CLN backend for the requested amount.

        The description is set to the raw metadata string so the client's
        BOLT11 description-hash check passes (as mandated by LUD-06).
        """
        self.pay_callbacks.append({"amount_msat": amount_msat, "comment": comment})

        # CLN requires a unique label per invoice
        label = f"lnurl-pay-{secrets.token_hex(8)}"

        # LUD-06: the BOLT11 description hash must equal SHA256(metadata).
        # pyln's `invoice` accepts `description` as a string; when CLN
        # encodes the invoice it hashes that string into the description
        # hash field, so passing our raw metadata JSON matches what the
        # client re-computes.
        invoice = self.cln_rpc.invoice(
            amount_msat=amount_msat,
            label=label,
            description=self.metadata,
            deschashonly=True,
        )

        response = {
            "pr": invoice["bolt11"],
            "routes": [],
        }
        if self.success_action is not None:
            response["successAction"] = self.success_action
        return response

    def build_withdraw_response(self, callback_path: str) -> dict:
        k1 = secrets.token_hex(16)
        self._pending_withdrawals[k1] = {"used": False}
        return {
            "tag": "withdrawRequest",
            "callback": f"{self.base_url}{callback_path}",
            "k1": k1,
            "defaultDescription": f"Withdraw from {self.domain}",
            "minWithdrawable": self.min_withdrawable,
            "maxWithdrawable": self.max_withdrawable,
        }

    def handle_withdraw_callback(self, k1: str, invoice: str) -> dict:
        """Pay the supplied BOLT11 invoice from the CLN backend."""
        self.withdraw_callbacks.append({"k1": k1, "pr": invoice})

        session = self._pending_withdrawals.get(k1)
        if session is None:
            return {"status": "ERROR", "reason": f"unknown k1: {k1}"}
        if session["used"]:
            return {"status": "ERROR", "reason": "k1 already used"}
        session["used"] = True

        try:
            self.cln_rpc.pay(invoice)
        except Exception as e:
            return {"status": "ERROR", "reason": f"pay failed: {e}"}

        return {"status": "OK"}


def _handler_factory(server: LnurlServer):
    """Build a BaseHTTPRequestHandler class bound to a specific server.

    Using a closure avoids squirreling state onto the ThreadingHTTPServer
    itself (as grpcweb.py does) and keeps the handler readable.
    """

    class _Handler(BaseHTTPRequestHandler):
        def log_message(self, format, *args):
            server.logger.debug("%s - - %s" % (self.address_string(), format % args))

        def _reply_json(self, code: int, payload: dict):
            body = json.dumps(payload).encode("utf-8")
            self.send_response(code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            parsed = urlparse(self.path)
            path = parsed.path
            query = parse_qs(parsed.query)

            try:
                if path == "/lnurlp":
                    self._reply_json(200, server.build_pay_response("/lnurlp/callback"))
                    return

                if path == f"/.well-known/lnurlp/{server.username}":
                    # LUD-16: use a different callback path so tests can
                    # distinguish address vs raw-lnurl code paths if they
                    # want to.
                    self._reply_json(
                        200,
                        server.build_pay_response(
                            f"/.well-known/lnurlp/{server.username}/callback"
                        ),
                    )
                    return

                if path in ("/lnurlp/callback", f"/.well-known/lnurlp/{server.username}/callback"):
                    amount = query.get("amount", [None])[0]
                    if amount is None:
                        self._reply_json(
                            200,
                            {"status": "ERROR", "reason": "missing amount"},
                        )
                        return
                    comment = query.get("comment", [None])[0]
                    self._reply_json(
                        200,
                        server.handle_pay_callback(int(amount), comment),
                    )
                    return

                if path == "/lnurlw":
                    self._reply_json(
                        200, server.build_withdraw_response("/lnurlw/callback")
                    )
                    return

                if path == "/lnurlw/callback":
                    k1 = query.get("k1", [None])[0]
                    pr = query.get("pr", [None])[0]
                    if not k1 or not pr:
                        self._reply_json(
                            200, {"status": "ERROR", "reason": "missing k1 or pr"}
                        )
                        return
                    self._reply_json(200, server.handle_withdraw_callback(k1, pr))
                    return

                self.send_response(404)
                self.end_headers()
            except Exception as e:
                server.logger.exception("Unhandled error in LnurlServer handler")
                self._reply_json(500, {"status": "ERROR", "reason": str(e)})

    return _Handler


def metadata_sha256(metadata: str) -> str:
    """Helper for tests that want to assert on description-hash matching."""
    return hashlib.sha256(metadata.encode("utf-8")).hexdigest()
