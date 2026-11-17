package com.blockstream.glsdk

import androidx.test.platform.app.InstrumentationRegistry
import java.nio.file.Files

internal fun signerStoragePath(): String =
    Files.createTempDirectory(
        InstrumentationRegistry.getInstrumentation().targetContext.filesDir.toPath(),
        "glsdk-signer-",
    ).toString()

internal fun signerConfig(): Config = Config().withSignerConfig(
    SignerConfig(StateSignatureMode.HARD, signerStoragePath()),
)
