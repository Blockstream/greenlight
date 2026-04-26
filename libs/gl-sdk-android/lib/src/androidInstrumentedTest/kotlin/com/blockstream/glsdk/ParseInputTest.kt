// Instrumented tests for the synchronous parse_input().
// `parse_input` is offline — no HTTP, no I/O. LNURL HTTP resolution
// is `resolveInput` and is covered by gl-testing integration tests.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class ParseInputTest {

    // Valid compressed secp256k1 public key
    private val validNodeId =
        "02eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619"

    // Valid BOLT11 invoice (11 sats, mainnet)
    private val bolt11Invoice =
        "lnbc110n1p38q3gtpp5ypz09jrd8p993snjwnm68cph4ftwp22le34xd4r8ftspwshxhm" +
        "nsdqqxqyjw5qcqpxsp5htlg8ydpywvsa7h3u4hdn77ehs4z4e844em0apjyvmqfkzqhh" +
        "d2q9qgsqqqyssqszpxzxt9uuqzymr7zxcdccj5g69s8q7zzjs7sgxn9ejhnvdh6gqjcy" +
        "22mss2yexunagm5r2gqczh8k24cwrqml3njskm548aruhpwssq9nvrvz"

    // Bech32-encoded "https://service.com/lnurl"
    private val lnurlBech32 =
        "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2"

    // ============================================================
    // Node ID parsing
    // ============================================================

    @Test
    fun parse_valid_node_id() {
        val result = parseInput(validNodeId)
        assertTrue("Expected NodeId, got $result", result is ParsedInput.NodeId)
    }

    @Test(expected = Exception::class)
    fun parse_invalid_hex_returns_error() {
        parseInput("not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx")
    }

    @Test(expected = Exception::class)
    fun parse_wrong_prefix_returns_error() {
        parseInput("04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")
    }

    // ============================================================
    // BOLT11 parsing
    // ============================================================

    @Test
    fun parse_valid_bolt11() {
        val result = parseInput(bolt11Invoice)
        assertTrue("Expected Bolt11, got $result", result is ParsedInput.Bolt11)
    }

    @Test
    fun parse_bolt11_with_lightning_prefix() {
        val result = parseInput("lightning:$bolt11Invoice")
        assertTrue("Expected Bolt11, got $result", result is ParsedInput.Bolt11)
    }

    @Test
    fun parse_bolt11_with_uppercase_prefix() {
        val result = parseInput("LIGHTNING:$bolt11Invoice")
        assertTrue("Expected Bolt11, got $result", result is ParsedInput.Bolt11)
    }

    // ============================================================
    // LNURL bech32 / Lightning Address — offline classification
    // ============================================================

    @Test
    fun parse_lnurl_bech32_decodes_url() {
        val result = parseInput(lnurlBech32)
        assertTrue("Expected LnUrl, got $result", result is ParsedInput.LnUrl)
        val url = (result as ParsedInput.LnUrl).url
        assertTrue("Expected decoded https URL, got $url", url.startsWith("https://"))
    }

    @Test
    fun parse_lightning_address() {
        val result = parseInput("user@example.com")
        assertTrue("Expected LnUrlAddress, got $result", result is ParsedInput.LnUrlAddress)
        assertEquals("user@example.com", (result as ParsedInput.LnUrlAddress).address)
    }

    // ============================================================
    // Error cases
    // ============================================================

    @Test(expected = Exception::class)
    fun parse_empty_string_returns_error() {
        parseInput("")
    }

    @Test(expected = Exception::class)
    fun parse_garbage_returns_error() {
        parseInput("hello world")
    }
}
