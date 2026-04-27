// SDK logging — apps install a LogListener to receive log output
// emitted by this SDK and by the underlying gl-client library.
//
// The bridge sits on top of the `log` crate facade: gl-sdk's own
// `tracing` calls are routed via `tracing`'s `log` feature, and
// gl-client's direct `log::*!` calls flow through the same channel.

use crate::Error;
use log::{Level, LevelFilter, Log, Metadata, Record};

/// Log level for filtering messages.
#[derive(Clone, Copy, Debug, uniffi::Enum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<Level> for LogLevel {
    fn from(level: Level) -> Self {
        match level {
            Level::Error => LogLevel::Error,
            Level::Warn => LogLevel::Warn,
            Level::Info => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
            Level::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

/// A single log message from the SDK.
#[derive(Clone, Debug, uniffi::Record)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    /// The module that produced this log (e.g. "gl_client::scheduler").
    pub target: String,
    /// Source file path, if the log macro recorded one.
    pub file: Option<String>,
    /// Source line number, if the log macro recorded one.
    pub line: Option<u32>,
}

/// Callback interface for receiving log messages.
///
/// `on_log` is invoked on the thread that emitted the log — which can
/// be any tokio worker or background thread inside the SDK. Keep the
/// implementation cheap and non-blocking; if you need UI updates,
/// hand the entry off to your app's main thread.
#[uniffi::export(callback_interface)]
pub trait LogListener: Send + Sync {
    fn on_log(&self, entry: LogEntry);
}

struct SdkLogger {
    listener: Box<dyn LogListener>,
}

impl Log for SdkLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Use the `log` crate's global max level so `set_log_level`
        // can change the filter at runtime. `log::max_level()` is an
        // atomic read that every log macro also consults.
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.listener.on_log(LogEntry {
                level: record.level().into(),
                message: record.args().to_string(),
                target: record.target().to_string(),
                file: record.file().map(|s| s.to_string()),
                line: record.line(),
            });
        }
    }

    fn flush(&self) {}
}

/// Install a log listener. Call this once, as early as possible, so
/// logs emitted during node bring-up are captured.
///
/// Returns `Err` if a logger is already installed in the process
/// (either from an earlier successful call to `set_logger` or from a
/// different crate). To change the filter after installation use
/// `set_log_level`.
pub fn set_logger(level: LogLevel, listener: Box<dyn LogListener>) -> Result<(), Error> {
    let filter: LevelFilter = level.into();
    log::set_boxed_logger(Box::new(SdkLogger { listener })).map_err(|e| {
        Error::Other(format!("a `log` logger is already installed: {e}"))
    })?;
    log::set_max_level(filter);
    Ok(())
}

/// Change the log filter at runtime without reinstalling the listener.
///
/// Useful for features like a "verbose logs" toggle in app settings.
/// Safe to call before `set_logger` — it adjusts the `log` crate's
/// global max level immediately; the listener (if any) picks it up
/// on the next emitted message.
pub fn set_log_level(level: LogLevel) {
    log::set_max_level(level.into());
}
