package com.blockstream.glsdk

import kotlin.test.Test

class CommonTest {

    @Test
    fun test_signer() {
        val mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        val signer = Signer(mnemonic)
        signer.start()
    }
}
