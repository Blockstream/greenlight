package com.blockstream.glsdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import glsdk.Signer
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Instrumented test, which will execute on an Android device.
 *
 * See [testing documentation](http://d.android.com/tools/testing).
 */
@RunWith(AndroidJUnit4::class)
class InstrumentedTest {

    @Test
    fun test() {
        val mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        val signer = Signer(mnemonic)
        signer.start()
    }
}