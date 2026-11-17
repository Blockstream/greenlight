package com.blockstream.glsdk

import kotlin.random.Random
import kotlin.test.Test
import okio.FileSystem

class NativeTest {

    @Test
    fun test_signer() {
        val mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        val path = FileSystem.SYSTEM_TEMPORARY_DIRECTORY / "glsdk-signer-${Random.nextLong()}"
        val signer = Signer(mnemonic, path.toString())
        signer.start()
    }
}
