"""Tests for the CLN-backed LNURL mock server.

These tests verify the server itself works correctly (invoice generation,
callback handling, etc.) before we layer in gl-client/gl-sdk tests that
depend on it.
"""

from gltesting.fixtures import *  # noqa: F401, F403
from gltesting.lnurl_server import metadata_sha256

import httpx
import json


def test_pay_request_response(lnurl_service):
    """GET /lnurlp returns a valid payRequest response."""
    r = httpx.get(lnurl_service.pay_url)
    assert r.status_code == 200
    body = r.json()

    assert body["tag"] == "payRequest"
    assert body["callback"].startswith(lnurl_service.base_url)
    assert body["minSendable"] == lnurl_service.min_sendable
    assert body["maxSendable"] == lnurl_service.max_sendable
    # metadata must parse as a JSON array of [mime, content] pairs
    meta = json.loads(body["metadata"])
    assert any(entry[0] == "text/plain" for entry in meta)


def test_lightning_address_endpoint(lnurl_service):
    """LUD-16: GET /.well-known/lnurlp/{user} returns a payRequest."""
    r = httpx.get(lnurl_service.lightning_address_url)
    assert r.status_code == 200
    body = r.json()
    assert body["tag"] == "payRequest"
    # Metadata must include a text/identifier entry for LUD-16
    meta = json.loads(body["metadata"])
    assert any(
        entry[0] == "text/identifier" and entry[1] == f"{lnurl_service.username}@{lnurl_service.domain}:{lnurl_service.port}"
        for entry in meta
    )


def test_pay_callback_returns_valid_invoice(lnurl_service):
    """GET /lnurlp/callback?amount=X returns a BOLT11 with the right
    description hash and amount."""
    # Fetch the pay request to get the callback URL
    pay_req = httpx.get(lnurl_service.pay_url).json()
    amount_msat = 10_000

    # Call the callback
    r = httpx.get(pay_req["callback"], params={"amount": amount_msat})
    assert r.status_code == 200
    body = r.json()
    assert "pr" in body

    # Decode the BOLT11 using the backing CLN node
    decoded = lnurl_service.cln_rpc.decodepay(body["pr"])
    assert decoded["amount_msat"] == amount_msat
    # description hash must match SHA256(metadata)
    expected_hash = metadata_sha256(pay_req["metadata"])
    assert decoded["description_hash"] == expected_hash


def test_pay_callback_tracked_in_callbacks_list(lnurl_service):
    """The server records every callback invocation for test inspection."""
    pay_req = httpx.get(lnurl_service.pay_url).json()
    httpx.get(pay_req["callback"], params={"amount": 5_000, "comment": "hello"})

    assert len(lnurl_service.pay_callbacks) == 1
    assert lnurl_service.pay_callbacks[0]["amount_msat"] == 5_000
    assert lnurl_service.pay_callbacks[0]["comment"] == "hello"


def test_withdraw_request_response(lnurl_service):
    """GET /lnurlw returns a valid withdrawRequest response with a fresh k1."""
    r = httpx.get(lnurl_service.withdraw_url)
    assert r.status_code == 200
    body = r.json()

    assert body["tag"] == "withdrawRequest"
    assert body["callback"].startswith(lnurl_service.base_url)
    assert len(body["k1"]) > 0
    assert body["minWithdrawable"] == lnurl_service.min_withdrawable
    assert body["maxWithdrawable"] == lnurl_service.max_withdrawable


def test_withdraw_issues_distinct_k1s(lnurl_service):
    """Each call to /lnurlw returns a fresh k1."""
    r1 = httpx.get(lnurl_service.withdraw_url).json()
    r2 = httpx.get(lnurl_service.withdraw_url).json()
    assert r1["k1"] != r2["k1"]
