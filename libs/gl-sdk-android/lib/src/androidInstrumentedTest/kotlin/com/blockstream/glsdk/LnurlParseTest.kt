// Instrumented tests for parse_input() error-before-HTTP cases on
// LNURL / Lightning Address inputs and the LUD-04 tag=login fast path.
//
// Successful resolution of LNURL-pay / LNURL-withdraw requires a
// reachable LNURL service and is covered by gl-testing integration
// tests, not by Android instrumented tests.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LnurlParseTest {

    // 32 zero bytes — a syntactically valid k1.
    private val zeroK1 =
        "0000000000000000000000000000000000000000000000000000000000000000"

    @Test(expected = Exception::class)
    fun parse_invalid_lnurl_bech32_returns_error(): Unit = runBlocking {
        parseInput("LNURL1INVALIDDATA")
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_no_dot_in_domain_returns_error(): Unit = runBlocking {
        parseInput("user@localhost")
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_empty_local_part_returns_error(): Unit = runBlocking {
        parseInput("@example.com")
    }

    @Test(expected = Exception::class)
    fun parse_lightning_address_empty_domain_returns_error(): Unit = runBlocking {
        parseInput("user@")
    }

    // ============================================================
    // LUD-04 tag=login — classified offline (no HTTP fetch)
    // ============================================================

    @Test
    fun parse_lnurl_auth_url_classifies_as_lnurlauth() = runBlocking {
        val url = "https://service.example.com/auth?tag=login&k1=$zeroK1"
        val result = parseInput(url)
        assertTrue(
            "Expected LnUrlAuth, got $result",
            result is InputType.LnUrlAuth,
        )
        val data = (result as InputType.LnUrlAuth).data
        assertEquals(zeroK1, data.k1)
        assertEquals("service.example.com", data.domain)
        assertNull(data.action)
        assertEquals(url, data.url)
    }

    @Test
    fun parse_lnurl_auth_url_captures_action() = runBlocking {
        val url = "https://x.com/a?tag=login&k1=$zeroK1&action=register"
        val result = parseInput(url)
        assertTrue(result is InputType.LnUrlAuth)
        assertEquals("register", (result as InputType.LnUrlAuth).data.action)
    }

    @Test(expected = Exception::class)
    fun parse_lnurl_auth_rejects_missing_k1(): Unit = runBlocking {
        parseInput("https://x.com/a?tag=login")
    }

    @Test(expected = Exception::class)
    fun parse_lnurl_auth_rejects_short_k1(): Unit = runBlocking {
        parseInput("https://x.com/a?tag=login&k1=deadbeef")
    }

    @Test(expected = Exception::class)
    fun parse_lnurl_auth_rejects_unknown_action(): Unit = runBlocking {
        parseInput("https://x.com/a?tag=login&k1=$zeroK1&action=bogus")
    }
}
