"""Tests for the async parse_input() free function.

Covers BOLT11 invoice and node ID parsing — paths that complete without
HTTP. LNURL and Lightning Address parsing are exercised in
test_lnurl.py against the live LNURL fixture.
"""

import asyncio

import pytest
import glsdk


# Valid BOLT11 invoice (11 sats, mainnet)
BOLT11_INVOICE = (
    "lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhm"
    "nsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhh"
    "d2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy"
    "22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz"
)

# Valid compressed secp256k1 public key (starts with 02 or 03, 33 bytes = 66 hex chars)
VALID_NODE_ID = "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619"


def parse(input_str):
    return asyncio.run(glsdk.parse_input(input_str))


class TestParseInputTypes:
    """Test that parse_input types exist in the bindings."""

    def test_input_type_enum_exists(self):
        assert hasattr(glsdk, "InputType")

    def test_bolt11_invoice_type_exists(self):
        assert hasattr(glsdk, "ParsedInvoice")

    def test_parse_input_function_exists(self):
        assert hasattr(glsdk, "parse_input")


class TestParseInputNodeId:
    """Test node ID parsing — no HTTP required."""

    def test_parse_valid_node_id(self):
        result = parse(VALID_NODE_ID)
        assert isinstance(result, glsdk.InputType.NODE_ID)
        assert result.node_id == VALID_NODE_ID

    def test_invalid_hex_returns_error(self):
        with pytest.raises(glsdk.Error):
            parse("not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx")

    def test_wrong_length_hex_returns_error(self):
        with pytest.raises(glsdk.Error):
            parse("02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283")

    def test_wrong_prefix_hex_returns_error(self):
        # 04 prefix = uncompressed pubkey, not valid for Lightning
        with pytest.raises(glsdk.Error):
            parse("04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")


class TestParseInputBolt11:
    """Test BOLT11 invoice parsing — no HTTP required."""

    def test_parse_valid_bolt11(self):
        result = parse(BOLT11_INVOICE)
        assert isinstance(result, glsdk.InputType.BOLT11)

    def test_parse_bolt11_with_lightning_prefix(self):
        result = parse("lightning:" + BOLT11_INVOICE)
        assert isinstance(result, glsdk.InputType.BOLT11)

    def test_parse_bolt11_with_uppercase_prefix(self):
        result = parse("LIGHTNING:" + BOLT11_INVOICE)
        assert isinstance(result, glsdk.InputType.BOLT11)

    def test_parse_bolt11_with_whitespace(self):
        result = parse("  " + BOLT11_INVOICE + "  ")
        assert isinstance(result, glsdk.InputType.BOLT11)


class TestParseInputErrors:
    """Test error cases that don't require HTTP."""

    def test_empty_string_returns_error(self):
        with pytest.raises(glsdk.Error):
            parse("")

    def test_whitespace_only_returns_error(self):
        with pytest.raises(glsdk.Error):
            parse("   ")

    def test_garbage_returns_error(self):
        with pytest.raises(glsdk.Error):
            parse("hello world")

    def test_bitcoin_address_returns_error(self):
        # We don't support bitcoin addresses yet
        with pytest.raises(glsdk.Error):
            parse("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")

    def test_invalid_lnurl_bech32_errors_before_http(self):
        with pytest.raises(glsdk.Error):
            parse("LNURL1INVALIDDATA")
