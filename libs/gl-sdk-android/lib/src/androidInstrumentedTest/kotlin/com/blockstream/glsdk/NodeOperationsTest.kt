package com.blockstream.glsdk

import android.system.Os
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import kotlin.uuid.ExperimentalUuidApi
import kotlin.uuid.Uuid

@RunWith(AndroidJUnit4::class)
class NodeOperationsTest {

    @Before
    fun setup() {
        Os.setenv("RUST_LOG", "trace", true)
    }

    // BIP39 test vector — not a real wallet
    private val testMnemonic =
        "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"

    @OptIn(ExperimentalUuidApi::class)
    @Test
    fun test_onchain_receive_and_invoice() {
        val config = Config()

        val node = NodeBuilder(config).registerOrRecover(testMnemonic, null)

        node.use { n ->
            // Get an on-chain address to fund the node
            val addrResponse = n.onchainReceive()
            println("Deposit funds to: ${addrResponse.toString()}")

            // Create a lightning invoice for 1000 sats (1,000,000 msats)
            val invoice = n.receive(
                label = Uuid.random().toString(),
                description = "Coffee",
                amountMsat = 20_000_000uL
            )
            println("Lightning Invoice: ${invoice.toString()}")
        }
    }

    @Test
    fun test_node_state_returns_valid_snapshot() {
        val config = Config()
        val node = NodeBuilder(config).registerOrRecover(testMnemonic, null)
        node.use { n ->
            val state = n.nodeState()
            assertTrue(state.id.isNotEmpty())
            assertTrue(state.blockHeight > 0u)
            assertEquals("bitcoin", state.network)
            assertTrue(state.version.isNotEmpty())
            assertEquals(0uL, state.channelsBalanceMsat)
            assertEquals(0uL, state.maxPayableMsat)
            assertEquals(0uL, state.totalChannelCapacityMsat)
            assertEquals(0uL, state.onchainBalanceMsat)
            assertEquals(0uL, state.totalInboundLiquidityMsat)
        }
    }
}
