"""Tests for the SDK logging framework.

`set_logger` installs a process-wide logger on the `log` crate — only
one install succeeds per process. Tests that exercise the install path
are ordered / deduped so we don't cascade failures.
"""

import glsdk


class TestLoggingTypes:
    """Type surface exists in the bindings."""

    def test_log_level_enum(self):
        assert hasattr(glsdk, "LogLevel")
        for variant in ("ERROR", "WARN", "INFO", "DEBUG", "TRACE"):
            assert hasattr(glsdk.LogLevel, variant)

    def test_log_entry_type(self):
        assert hasattr(glsdk, "LogEntry")

    def test_log_listener_type(self):
        assert hasattr(glsdk, "LogListener")

    def test_set_logger_function(self):
        assert hasattr(glsdk, "set_logger")

    def test_set_log_level_function(self):
        assert hasattr(glsdk, "set_log_level")


class TestLogEntryShape:
    """LogEntry carries level / message / target / file / line."""

    def test_log_entry_fields(self):
        entry = glsdk.LogEntry(
            level=glsdk.LogLevel.INFO,
            message="hello",
            target="gl_sdk::test",
            file="src/test.rs",
            line=42,
        )
        assert entry.level == glsdk.LogLevel.INFO
        assert entry.message == "hello"
        assert entry.target == "gl_sdk::test"
        assert entry.file == "src/test.rs"
        assert entry.line == 42

    def test_log_entry_file_line_optional(self):
        entry = glsdk.LogEntry(
            level=glsdk.LogLevel.ERROR,
            message="oops",
            target="gl_sdk",
            file=None,
            line=None,
        )
        assert entry.file is None
        assert entry.line is None


class TestSetLogLevel:
    """`set_log_level` is callable without a listener installed."""

    def test_set_log_level_no_raise(self):
        # Safe to call before/after / without set_logger
        glsdk.set_log_level(glsdk.LogLevel.WARN)
        glsdk.set_log_level(glsdk.LogLevel.TRACE)
