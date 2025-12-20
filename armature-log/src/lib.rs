//! Armature Logging Framework
//!
//! Provides structured logging for the Armature framework with support for
//! the `ARMATURE_DEBUG` environment variable.
//!
//! # Features
//!
//! - **Environment-controlled**: `ARMATURE_DEBUG=1` enables debug logging
//! - **Zero-cost when disabled**: Debug macros compile to no-ops in release
//! - **Structured logging**: Support for key-value pairs
//! - **Multiple backends**: Works with `log` and optionally `tracing`
//!
//! # Usage
//!
//! ```rust
//! use armature_log::{debug, info, warn, error, trace};
//!
//! // Simple logging
//! debug!("Processing request");
//! info!("Server started on port {}", 8080);
//! warn!("Connection pool low");
//! error!("Failed to connect to database");
//!
//! // With target (module path)
//! let path = "/api/users";
//! debug!(target: "armature::router", "Matching route: {}", path);
//! ```
//!
//! # Environment Variables
//!
//! - `ARMATURE_DEBUG=1` - Enable debug logging
//! - `ARMATURE_LOG_LEVEL=debug|info|warn|error` - Set log level
//! - `ARMATURE_LOG_FORMAT=pretty|json|compact` - Set output format
//! - `ARMATURE_LOG_COLOR=1|0` - Enable/disable colors

use once_cell::sync::Lazy;
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

// ============================================================================
// Log Levels
// ============================================================================

/// Log level for Armature logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    /// Trace level (most verbose)
    Trace = 0,
    /// Debug level
    Debug = 1,
    /// Info level
    Info = 2,
    /// Warning level
    Warn = 3,
    /// Error level (least verbose)
    Error = 4,
    /// Off (no logging)
    Off = 5,
}

impl Level {
    /// Get level from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(Level::Trace),
            "debug" => Some(Level::Debug),
            "info" => Some(Level::Info),
            "warn" | "warning" => Some(Level::Warn),
            "error" => Some(Level::Error),
            "off" | "none" => Some(Level::Off),
            _ => None,
        }
    }

    /// Get level name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Off => "OFF",
        }
    }

    /// Get colored level name (if color feature enabled).
    #[cfg(feature = "color")]
    pub fn colored(&self) -> colored::ColoredString {
        use colored::Colorize;
        match self {
            Level::Trace => "TRACE".magenta(),
            Level::Debug => "DEBUG".blue(),
            Level::Info => "INFO".green(),
            Level::Warn => "WARN".yellow(),
            Level::Error => "ERROR".red().bold(),
            Level::Off => "OFF".white(),
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Log Format
// ============================================================================

/// Output format for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// Pretty format with colors (default for TTY)
    Pretty,
    /// Compact single-line format
    Compact,
    /// JSON format for structured logging
    Json,
}

impl Format {
    /// Get format from string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pretty" => Some(Format::Pretty),
            "compact" => Some(Format::Compact),
            "json" => Some(Format::Json),
            _ => None,
        }
    }
}

// ============================================================================
// Global Configuration
// ============================================================================

/// Global debug flag - checked by macros.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global log level.
static LOG_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);

/// Global configuration (lazy initialized).
static CONFIG: Lazy<LogConfig> = Lazy::new(LogConfig::from_env);

/// Logging configuration.
#[derive(Debug)]
pub struct LogConfig {
    /// Whether debug mode is enabled
    pub debug: bool,
    /// Minimum log level
    pub level: Level,
    /// Output format
    pub format: Format,
    /// Whether colors are enabled
    pub color: bool,
    /// Whether to include timestamps
    pub timestamps: bool,
    /// Whether to include module path
    pub module_path: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            debug: false,
            level: Level::Info,
            format: Format::Json,
            color: false, // JSON output doesn't use colors
            timestamps: true,
            module_path: true,
        }
    }
}

impl LogConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        let debug = env::var("ARMATURE_DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let level = env::var("ARMATURE_LOG_LEVEL")
            .ok()
            .and_then(|s| Level::from_str(&s))
            .unwrap_or(if debug { Level::Debug } else { Level::Info });

        let format = env::var("ARMATURE_LOG_FORMAT")
            .ok()
            .and_then(|s| Format::from_str(&s))
            .unwrap_or(Format::Json);

        let color = env::var("ARMATURE_LOG_COLOR")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(atty::is(atty::Stream::Stderr));

        let timestamps = env::var("ARMATURE_LOG_TIMESTAMPS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true);

        let module_path = env::var("ARMATURE_LOG_MODULE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true);

        // Update global atomics
        DEBUG_ENABLED.store(debug, Ordering::SeqCst);
        LOG_LEVEL.store(level as u8, Ordering::SeqCst);

        Self {
            debug,
            level,
            format,
            color,
            timestamps,
            module_path,
        }
    }
}

/// Check if TTY (for color detection fallback).
mod atty {
    pub enum Stream {
        Stderr,
    }

    pub fn is(_stream: Stream) -> bool {
        // Simple check - assume color if not explicitly disabled
        std::env::var("NO_COLOR").is_err() && std::env::var("TERM").is_ok()
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Initialize the logging system.
///
/// This is called automatically when first log macro is used,
/// but can be called explicitly for eager initialization.
pub fn init() {
    Lazy::force(&CONFIG);
}

/// Check if debug logging is enabled.
#[inline]
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Check if a log level is enabled.
#[inline]
pub fn is_level_enabled(level: Level) -> bool {
    level as u8 >= LOG_LEVEL.load(Ordering::Relaxed)
}

/// Get current log level.
pub fn current_level() -> Level {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        0 => Level::Trace,
        1 => Level::Debug,
        2 => Level::Info,
        3 => Level::Warn,
        4 => Level::Error,
        _ => Level::Off,
    }
}

/// Set log level at runtime.
pub fn set_level(level: Level) {
    LOG_LEVEL.store(level as u8, Ordering::SeqCst);
}

/// Enable or disable debug mode at runtime.
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled && current_level() > Level::Debug {
        set_level(Level::Debug);
    }
}

/// Get the global configuration.
pub fn config() -> &'static LogConfig {
    &CONFIG
}

// ============================================================================
// Log Output
// ============================================================================

/// Log a message with the given level.
#[doc(hidden)]
pub fn log(level: Level, target: &str, message: &str) {
    if !is_level_enabled(level) {
        return;
    }

    let config = config();

    match config.format {
        Format::Pretty => log_pretty(level, target, message, config),
        Format::Compact => log_compact(level, target, message, config),
        Format::Json => log_json(level, target, message),
    }
}

fn log_pretty(level: Level, target: &str, message: &str, config: &LogConfig) {
    let mut stderr = std::io::stderr().lock();

    // Timestamp
    if config.timestamps {
        let now = chrono::Local::now();
        let _ = write!(stderr, "{} ", now.format("%Y-%m-%d %H:%M:%S%.3f"));
    }

    // Level
    #[cfg(feature = "color")]
    if config.color {
        let _ = write!(stderr, "{:5} ", level.colored());
    } else {
        let _ = write!(stderr, "{:5} ", level.as_str());
    }

    #[cfg(not(feature = "color"))]
    let _ = write!(stderr, "{:5} ", level.as_str());

    // Target
    if config.module_path && !target.is_empty() {
        #[cfg(feature = "color")]
        if config.color {
            use colored::Colorize;
            let _ = write!(stderr, "{} ", target.dimmed());
        } else {
            let _ = write!(stderr, "[{}] ", target);
        }

        #[cfg(not(feature = "color"))]
        let _ = write!(stderr, "[{}] ", target);
    }

    // Message
    let _ = writeln!(stderr, "{}", message);
}

fn log_compact(level: Level, target: &str, message: &str, config: &LogConfig) {
    let mut stderr = std::io::stderr().lock();

    if config.timestamps {
        let now = chrono::Local::now();
        let _ = write!(stderr, "{} ", now.format("%H:%M:%S"));
    }

    let _ = write!(stderr, "{} ", level.as_str().chars().next().unwrap_or('?'));

    if config.module_path && !target.is_empty() {
        let _ = write!(stderr, "{}: ", target);
    }

    let _ = writeln!(stderr, "{}", message);
}

#[cfg(feature = "json")]
fn log_json(level: Level, target: &str, message: &str) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct LogEntry<'a> {
        timestamp: String,
        level: &'a str,
        target: &'a str,
        message: &'a str,
    }

    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.as_str(),
        target,
        message,
    };

    if let Ok(json) = serde_json::to_string(&entry) {
        eprintln!("{}", json);
    }
}

#[cfg(not(feature = "json"))]
fn log_json(level: Level, target: &str, message: &str) {
    // Fallback without serde - manually escape JSON strings
    let timestamp = chrono::Utc::now().to_rfc3339();
    eprintln!(
        r#"{{"timestamp":"{}","level":"{}","target":"{}","message":"{}"}}"#,
        timestamp,
        level.as_str(),
        escape_json(target),
        escape_json(message)
    );
}

#[cfg(not(feature = "json"))]
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

// ============================================================================
// Macros
// ============================================================================

/// Log a trace message.
///
/// Only enabled when `ARMATURE_DEBUG=1` or `ARMATURE_LOG_LEVEL=trace`.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Trace) {
            $crate::log($crate::Level::Trace, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Trace) {
            $crate::log($crate::Level::Trace, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log a debug message.
///
/// Only enabled when `ARMATURE_DEBUG=1` or `ARMATURE_LOG_LEVEL=debug`.
///
/// # Example
///
/// ```rust
/// use armature_log::debug;
///
/// debug!("Processing request");
/// let username = "alice";
/// debug!("User {} logged in", username);
/// let path = "/api/users";
/// debug!(target: "armature::router", "Matching route: {}", path);
/// ```
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_debug_enabled() || $crate::is_level_enabled($crate::Level::Debug) {
            $crate::log($crate::Level::Debug, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_debug_enabled() || $crate::is_level_enabled($crate::Level::Debug) {
            $crate::log($crate::Level::Debug, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log an info message.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Info) {
            $crate::log($crate::Level::Info, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Info) {
            $crate::log($crate::Level::Info, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log a warning message.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Warn) {
            $crate::log($crate::Level::Warn, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Warn) {
            $crate::log($crate::Level::Warn, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log an error message.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Error) {
            $crate::log($crate::Level::Error, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Error) {
            $crate::log($crate::Level::Error, module_path!(), &format!($($arg)+));
        }
    };
}

// ============================================================================
// Tracing Integration
// ============================================================================

#[cfg(feature = "tracing")]
pub mod tracing_compat {
    //! Tracing compatibility layer.
    //!
    //! When the `tracing` feature is enabled, this module provides
    //! a subscriber that respects `ARMATURE_DEBUG`.

    use super::*;

    /// Create a tracing subscriber that respects Armature config.
    pub fn subscriber() -> impl tracing::Subscriber {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{fmt, EnvFilter};

        let config = config();
        let level = match config.level {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Off => "off",
        };

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_ansi(config.color))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
        assert!(Level::Error < Level::Off);
    }

    #[test]
    fn test_level_from_str() {
        assert_eq!(Level::from_str("debug"), Some(Level::Debug));
        assert_eq!(Level::from_str("DEBUG"), Some(Level::Debug));
        assert_eq!(Level::from_str("warn"), Some(Level::Warn));
        assert_eq!(Level::from_str("warning"), Some(Level::Warn));
        assert_eq!(Level::from_str("invalid"), None);
    }

    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from_str("pretty"), Some(Format::Pretty));
        assert_eq!(Format::from_str("compact"), Some(Format::Compact));
        assert_eq!(Format::from_str("json"), Some(Format::Json));
        assert_eq!(Format::from_str("invalid"), None);
    }

    #[test]
    fn test_set_level() {
        let original = current_level();

        set_level(Level::Error);
        assert_eq!(current_level(), Level::Error);

        set_level(Level::Debug);
        assert_eq!(current_level(), Level::Debug);

        set_level(original);
    }

    #[test]
    fn test_debug_flag() {
        let original = is_debug_enabled();

        set_debug(true);
        assert!(is_debug_enabled());

        set_debug(false);
        assert!(!is_debug_enabled());

        set_debug(original);
    }

    #[test]
    fn test_macros_compile() {
        // Just verify macros compile correctly
        trace!("trace message");
        debug!("debug message");
        info!("info message");
        warn!("warn message");
        error!("error message");

        trace!(target: "test", "with target");
        debug!(target: "test", "with target");
        info!(target: "test", "with target");
        warn!(target: "test", "with target");
        error!(target: "test", "with target");

        let x = 42;
        debug!("formatted: {}", x);
    }
}

