// Instrumented tests for the high-level auth API.
// Tests Config, register, recover, connect, and register_or_recover
// against the Greenlight production scheduler.

package com.blockstream.glsdk

import android.system.Os
import androidx.test.ext.junit.runners.AndroidJUnit4
import okio.ByteString.Companion.decodeBase64
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@RunWith(AndroidJUnit4::class)
class AuthApiTest {

    @Before
    fun setup() {
        Os.setenv("RUST_LOG", "trace", true)
    }

    // BIP39 test vector — not a real wallet
    private val testMnemonic =
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"

    // ============================================================
    // Config
    // ============================================================

    @Test
    fun config_default() {
        val config = Config()
        assertNotNull(config)
    }

    @Test
    fun config_with_network() {
        val config = Config().withNetwork(Network.REGTEST)
        assertNotNull(config)
    }

    @Test
    fun config_with_developer_cert() {
        val cert = DeveloperCert("fake-cert".toByteArray(), "fake-key".toByteArray())
        val config = Config().withDeveloperCert(cert)
        assertNotNull(config)
    }

    @Test
    fun config_builder_chaining() {
        val cert = DeveloperCert("fake-cert".toByteArray(), "fake-key".toByteArray())
        val config = Config()
            .withDeveloperCert(cert)
            .withNetwork(Network.REGTEST)
        assertNotNull(config)
    }

    // ============================================================
    // Bad mnemonic
    // ============================================================

    @Test(expected = Exception.PhraseCorrupted::class)
    fun register_bad_mnemonic() {
        val config = Config()
        NodeBuilder(config).register("not a valid mnemonic", null)
    }

    @Test(expected = Exception.PhraseCorrupted::class)
    fun recover_bad_mnemonic() {
        val config = Config()
        NodeBuilder(config).recover("not a valid mnemonic")
    }

    @Test(expected = Exception.PhraseCorrupted::class)
    fun connect_bad_mnemonic() {
        val config = Config()
        NodeBuilder(config).connect("fake-creds".toByteArray(), "not a valid mnemonic")
    }

    // ============================================================
    // Register and connect flow
    // ============================================================

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun register_or_recover_returns_node() {
        val config = Config()
        val node = NodeBuilder(config).registerOrRecover(testMnemonic, null)
        assertNotNull(node)
        node.use { n ->
            val creds = n.credentials()
            assertNotNull(creds)
            assertTrue("Credentials should not be empty", creds.isNotEmpty())
        }
    }

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun credentials_roundtrip_via_connect() {
        val config = Config()

        // Register or recover to get credentials
        val savedCreds: ByteArray
        NodeBuilder(config).registerOrRecover(testMnemonic, null).use { node ->
            savedCreds = node.credentials()
        }

        // Connect with the saved credentials
        NodeBuilder(config).connect(savedCreds, testMnemonic).use { node ->
            assertNotNull(node)
            val reconnectedCreds = node.credentials()
            assertTrue("Reconnected credentials should not be empty", reconnectedCreds.isNotEmpty())
        }
    }

    // ============================================================
    // Disconnect
    // ============================================================

    @Test
    fun disconnect_is_idempotent() {
        val config = Config()

        val node = NodeBuilder(config).registerOrRecover(testMnemonic, null)
        // First disconnect
        node.disconnect()
        // Second disconnect should not throw
        node.disconnect()
    }

    // ============================================================
    // Node operations
    // ============================================================

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun register_or_recover_and_create_invoice() {
        val config = Config()
        NodeBuilder(config).registerOrRecover(testMnemonic, null).use { node ->
            val addrResponse = node.onchainReceive()
            assertNotNull(addrResponse)
            println("Deposit funds to: $addrResponse")

            val invoice = node.receive(
                label = Uuid.random().toString(),
                description = "Coffee",
                amountMsat = 20_000_000uL
            )
            assertNotNull(invoice)
            assertTrue("Invoice bolt11 should not be empty", invoice.bolt11.isNotEmpty())
            println("Lightning Invoice: $invoice")
        }
    }

    // ============================================================
    // Low-level API still works
    // ============================================================

    @Test
    fun low_level_signer_still_available() {
        val signer = Signer(testMnemonic)
        assertNotNull(signer)
        val nodeId = signer.nodeId()
        assertTrue("Node ID should not be empty", nodeId.isNotEmpty())
    }

    @Test
    fun low_level_scheduler_still_available() {
        val scheduler = Scheduler(Network.BITCOIN)
        assertNotNull(scheduler)
    }
}
