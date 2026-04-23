// Instrumented tests for parse_input() LNURL handling.
// Covers LNURL bech32 strings, lightning addresses, prefix handling,
// and error cases. Pure parsing only — no node, no network.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LnurlParseTest {

    // LNURL bech32 encoding of https://service.com/lnurl (LUD-01 example).
    private val lnurlBech32 =
        "LNURL1DP68GURN8GHJ7CMFWP5X2UNSW4HXKTNRDAKJ7CTSDYHHVVF0D3H82UNV9UCSAXQZE2"

    // ============================================================
    // LNURL bech32 parsing
    // ============================================================

    @Test
    fun parse_lnurl_bech32_uppercase() {
        val result = parseInput(lnurlBech32)
        assertTrue(
            "Expected LnUrl, got $result",
            result is InputType.LnUrl,
        )
    }

    @Test
    fun parse_lnurl_bech32_lowercase() {
        val result = parseInput(lnurlBech32.lowercase())
        assertTrue(
            "Expected LnUrl, got $result",
            result is InputType.LnUrl,
        )
    }

    @Test
    fun parse_lnurl_with_lightning_prefix() {
        val result = parseInput("lightning:$lnurlBech32")
        assertTrue(
            "Expected LnUrl, got $result",
            result is InputType.LnUrl,
        )
    }

    @Test(expected = Exception::class)
    fun parse_invalid_lnurl_bech32_returns_error() {
        parseInput("LNURL1INVALIDDATA")
    }

    // ============================================================
    // Lightning address parsing
    // ============================================================

    @Test
    fun parse_lightning_address_simple() {
        val result = parseInput("user@example.com")
        assertTrue(
            "Expected LnUrlAddress, got $result",
            result is InputType.LnUrlAddress,
        )
        assertEquals("user@example.com", (result as InputType.LnUrlAddress).address)
    }

    @Test
    fun parse_lightning_address_with_symbols() {
        val result = parseInput("sat.oshi-99@example.com")
        assertTrue(
            "Expected LnUrlAddress, got $result",
            result is InputType.LnUrlAddress,
        )
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_no_dot_in_domain_returns_error() {
        parseInput("user@localhost")
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_empty_local_part_returns_error() {
        parseInput("@example.com")
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_empty_domain_returns_error() {
        parseInput("user@")
    }
}