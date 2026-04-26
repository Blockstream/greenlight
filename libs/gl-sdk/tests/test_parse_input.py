"""Tests for the synchronous parse_input() free function.

`parse_input` is offline — no HTTP, no I/O. Tests cover BOLT11
invoices, node IDs, and the offline LNURL / Lightning Address
classification (not the HTTP resolution — that's `resolve_input`,
exercised in test_lnurl.py against the live LNURL fixture).
"""

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

# Bech32-encoded "https://service.com/lnurl" (LUD-01 example)
VALID_LNURL_BECH32 = (
    "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2"
)


class TestParseInputTypes:
    """Test that parse_input types exist in the bindings."""

    def test_parsed_input_enum_exists(self):
        assert hasattr(glsdk, "ParsedInput")

    def test_resolved_input_enum_exists(self):
        assert hasattr(glsdk, "ResolvedInput")

    def test_bolt11_invoice_type_exists(self):
        assert hasattr(glsdk, "ParsedInvoice")

    def test_parse_input_function_exists(self):
        assert hasattr(glsdk, "parse_input")

    def test_resolve_input_function_exists(self):
        assert hasattr(glsdk, "resolve_input")


class TestParseInputNodeId:
    """Test node ID parsing — no HTTP required."""

    def test_parse_valid_node_id(self):
        result = glsdk.parse_input(VALID_NODE_ID)
        assert isinstance(result, glsdk.ParsedInput.NODE_ID)
        assert result.node_id == VALID_NODE_ID

    def test_invalid_hex_returns_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input(
                "not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx"
            )

    def test_wrong_length_hex_returns_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283")

    def test_wrong_prefix_hex_returns_error(self):
        # 04 prefix = uncompressed pubkey, not valid for Lightning
        with pytest.raises(glsdk.Error):
            glsdk.parse_input(
                "04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619"
            )


class TestParseInputBolt11:
    """Test BOLT11 invoice parsing — no HTTP required."""

    def test_parse_valid_bolt11(self):
        result = glsdk.parse_input(BOLT11_INVOICE)
        assert isinstance(result, glsdk.ParsedInput.BOLT11)

    def test_parse_bolt11_with_lightning_prefix(self):
        result = glsdk.parse_input("lightning:" + BOLT11_INVOICE)
        assert isinstance(result, glsdk.ParsedInput.BOLT11)

    def test_parse_bolt11_with_uppercase_prefix(self):
        result = glsdk.parse_input("LIGHTNING:" + BOLT11_INVOICE)
        assert isinstance(result, glsdk.ParsedInput.BOLT11)

    def test_parse_bolt11_with_whitespace(self):
        result = glsdk.parse_input("  " + BOLT11_INVOICE + "  ")
        assert isinstance(result, glsdk.ParsedInput.BOLT11)


class TestParseInputLnUrl:
    """Test LNURL bech32 / Lightning Address classification — offline."""

    def test_lnurl_bech32_decodes_to_url(self):
        result = glsdk.parse_input(VALID_LNURL_BECH32)
        assert isinstance(result, glsdk.ParsedInput.LN_URL)
        assert result.url.startswith("https://")

    def test_lnurl_bech32_lowercase(self):
        result = glsdk.parse_input(VALID_LNURL_BECH32.lower())
        assert isinstance(result, glsdk.ParsedInput.LN_URL)

    def test_lnurl_with_lightning_prefix(self):
        result = glsdk.parse_input("lightning:" + VALID_LNURL_BECH32)
        assert isinstance(result, glsdk.ParsedInput.LN_URL)

    def test_lightning_address_returns_address_form(self):
        result = glsdk.parse_input("user@example.com")
        assert isinstance(result, glsdk.ParsedInput.LN_URL_ADDRESS)
        assert result.address == "user@example.com"

    def test_lightning_address_with_symbols(self):
        # LUD-16 allows a-z0-9-_.
        result = glsdk.parse_input("sat.oshi-99@example.com")
        assert isinstance(result, glsdk.ParsedInput.LN_URL_ADDRESS)


class TestParseInputErrors:
    """Test error cases that don't require HTTP."""

    def test_empty_string_returns_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("")

    def test_whitespace_only_returns_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("   ")

    def test_garbage_returns_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("hello world")

    def test_bitcoin_address_returns_error(self):
        # We don't support bitcoin addresses yet
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4")

    def test_invalid_lnurl_bech32_errors(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("LNURL1INVALIDDATA")

    def test_lightning_address_no_dot_in_domain_errors(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("user@localhost")

    def test_lightning_address_empty_parts_error(self):
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("@example.com")
        with pytest.raises(glsdk.Error):
            glsdk.parse_input("user@")
