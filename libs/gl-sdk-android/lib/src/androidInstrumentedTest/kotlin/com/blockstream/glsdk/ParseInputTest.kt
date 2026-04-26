// Instrumented tests for parse_input().
// Tests BOLT11 invoice parsing, node ID parsing, and error cases that
// resolve without HTTP. LNURL / Lightning Address paths are exercised
// in gl-testing integration tests against a live LNURL fixture.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
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

    // ============================================================
    // Node ID parsing
    // ============================================================

    @Test
    fun parse_valid_node_id() = runBlocking {
        val result = parseInput(validNodeId)
        assertTrue(
            "Expected NodeId, got $result",
            result is InputType.NodeId,
        )
    }

    @Test(expected = Exception::class)
    fun parse_invalid_hex_returns_error(): Unit = runBlocking {
        parseInput("not_valid_hex_at_all_but_66_chars_long_xxxxxxxxxxxxxxxxxxxxxxxxxxx")
    }

    @Test(expected = Exception::class)
    fun parse_wrong_prefix_returns_error(): Unit = runBlocking {
        parseInput("04eec7245d6b7d2ccb30380bfbe2a3648cd7a942653f5aa340edcea1f283686619")
    }

    // ============================================================
    // BOLT11 parsing
    // ============================================================

    @Test
    fun parse_valid_bolt11() = runBlocking {
        val result = parseInput(bolt11Invoice)
        assertTrue(
            "Expected Bolt11, got $result",
            result is InputType.Bolt11,
        )
    }

    @Test
    fun parse_bolt11_with_lightning_prefix() = runBlocking {
        val result = parseInput("lightning:$bolt11Invoice")
        assertTrue(
            "Expected Bolt11, got $result",
            result is InputType.Bolt11,
        )
    }

    @Test
    fun parse_bolt11_with_uppercase_prefix() = runBlocking {
        val result = parseInput("LIGHTNING:$bolt11Invoice")
        assertTrue(
            "Expected Bolt11, got $result",
            result is InputType.Bolt11,
        )
    }

    // ============================================================
    // Error cases
    // ============================================================

    @Test(expected = Exception::class)
    fun parse_empty_string_returns_error(): Unit = runBlocking {
        parseInput("")
    }

    @Test(expected = Exception::class)
    fun parse_garbage_returns_error(): Unit = runBlocking {
        parseInput("hello world")
    }
}
