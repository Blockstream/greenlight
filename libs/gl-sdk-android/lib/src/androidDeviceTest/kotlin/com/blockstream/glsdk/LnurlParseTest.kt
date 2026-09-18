// Instrumented tests for parse_input() error cases on LNURL /
// Lightning Address inputs. `parse_input` is offline — these all
// reject before any HTTP would be attempted. Successful HTTP
// resolution is covered by `resolveInput` in gl-testing integration
// tests against a live LNURL fixture.

package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class LnurlParseTest {

    @Test(expected = Exception::class)
    fun parse_invalid_lnurl_bech32_returns_error() {
        parseInput("LNURL1INVALIDDATA")
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
