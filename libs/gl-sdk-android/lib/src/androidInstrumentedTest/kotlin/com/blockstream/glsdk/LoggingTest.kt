// Instrumented tests for the SDK logging framework.
//
// `setLogger` installs a process-wide logger — only one install
// succeeds per process. A `@Before` step installs a shared capturing
// listener; subsequent tests inspect it or call `setLogLevel` without
// re-installing.

package com.blockstream.glsdk

import android.util.Log
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.BeforeClass
import org.junit.Test
import org.junit.runner.RunWith
import java.util.concurrent.ConcurrentLinkedQueue

@RunWith(AndroidJUnit4::class)
class LoggingTest {

    companion object {
        private val captured = ConcurrentLinkedQueue<LogEntry>()

        @BeforeClass
        @JvmStatic
        fun installLogger() {
            val listener = object : LogListener {
                override fun onLog(entry: LogEntry) {
                    captured.add(entry)
                    Log.d("glsdk", "[${entry.level}] ${entry.target}: ${entry.message}")
                }
            }
            try {
                setLogger(LogLevel.TRACE, listener)
            } catch (e: Exception) {
                // Already installed by a prior test class in the same
                // process — expected when multiple test classes run.
            }
        }
    }

    @Test
    fun set_log_level_does_not_throw() {
        setLogLevel(LogLevel.WARN)
        setLogLevel(LogLevel.TRACE)
    }

    @Test
    fun listener_receives_logs_during_sdk_activity() {
        captured.clear()
        setLogLevel(LogLevel.TRACE)

        val config = Config()
        val mnemonic = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
        try {
            registerOrRecover(mnemonic, null, config)
        } catch (_: Exception) {
            // May fail on network / credentials — we only care that logs flowed.
        }

        val entries = captured.toList()
        Log.d("glsdk", "Captured ${entries.size} log entries")
        assertTrue(
            "Expected at least one log entry from gl-client during register_or_recover",
            entries.isNotEmpty(),
        )
        // Every entry should have a non-empty target and message
        for (entry in entries) {
            assertFalse("empty target in $entry", entry.target.isEmpty())
            assertNotNull(entry.message)
        }
    }
}
