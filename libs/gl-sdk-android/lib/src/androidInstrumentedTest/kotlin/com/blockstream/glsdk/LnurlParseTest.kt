// Instrumented tests for parse_input() error-before-HTTP cases on
// LNURL / Lightning Address inputs.
//
// Successful resolution requires a reachable LNURL service and is
// covered by gl-testing integration tests, not by Android instrumented
// tests.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import kotlinx.coroutines.runBlocking
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LnurlParseTest {

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
}
